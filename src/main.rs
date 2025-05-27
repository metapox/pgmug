use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod config;
mod oidc;
mod postgres;

use config::Config;
use oidc::OidcValidator;
use postgres::PostgresPool;

#[derive(Clone)]
pub struct AppState {
    pub postgres_pool: PostgresPool,
    pub oidc_validator: Arc<OidcValidator>,
}

#[derive(Deserialize)]
struct QueryRequest {
    sql: String,
}

#[derive(Serialize)]
struct QueryResponse {
    rows: Vec<serde_json::Value>,
    rows_affected: Option<u64>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "postgres-oidc-proxy"
    }))
}

async fn execute_query(
    State(state): State<AppState>,
    Json(query_req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let client = state
        .postgres_pool
        .get_client()
        .await
        .map_err(|e| {
            warn!("Failed to get database client: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database connection failed".to_string(),
                }),
            )
        })?;

    // Execute the query
    match client.query(&query_req.sql, &[]).await {
        Ok(rows) => {
            let mut result_rows = Vec::new();
            for row in rows {
                let mut json_row = serde_json::Map::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let value: Option<String> = row.try_get(i).ok();
                    json_row.insert(
                        column.name().to_string(),
                        value.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
                    );
                }
                result_rows.push(serde_json::Value::Object(json_row));
            }
            Ok(Json(QueryResponse {
                rows: result_rows,
                rows_affected: None,
            }))
        }
        Err(e) => {
            warn!("Query execution failed: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Query execution failed: {}", e),
                }),
            ))
        }
    }
}

async fn execute_mutation(
    State(state): State<AppState>,
    Json(query_req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let client = state
        .postgres_pool
        .get_client()
        .await
        .map_err(|e| {
            warn!("Failed to get database client: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database connection failed".to_string(),
                }),
            )
        })?;

    // Execute the mutation
    match client.execute(&query_req.sql, &[]).await {
        Ok(rows_affected) => Ok(Json(QueryResponse {
            rows: vec![],
            rows_affected: Some(rows_affected),
        })),
        Err(e) => {
            warn!("Mutation execution failed: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Mutation execution failed: {}", e),
                }),
            ))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // Initialize OIDC validator
    let oidc_validator = Arc::new(OidcValidator::new(&config.oidc).await?);
    info!("OIDC validator initialized");

    // Initialize PostgreSQL pool
    let postgres_pool = PostgresPool::new(&config.database).await?;
    info!("PostgreSQL connection pool initialized");

    let app_state = AppState {
        postgres_pool,
        oidc_validator,
    };

    // Build the application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/query", post(execute_query))
        .route("/execute", post(execute_mutation))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            oidc::auth_middleware,
        ))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&config.server.bind_address).await?;
    info!("Server starting on {}", config.server.bind_address);

    axum::serve(listener, app).await?;

    Ok(())
}
