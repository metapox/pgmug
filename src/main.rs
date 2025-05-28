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
    // Add early debugging output
    println!("=== Application starting... ===");
    eprintln!("=== Application starting (stderr)... ===");
    
    // Set panic hook for debugging
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC: {}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!("PANIC LOCATION: {}:{}:{}", location.file(), location.line(), location.column());
        }
    }));
    
    // Initialize tracing
    println!("Initializing tracing...");
    tracing_subscriber::fmt::init();
    
    println!("Tracing initialized");
    info!("Starting PostgreSQL OIDC Proxy");

    // Load configuration
    let config = match Config::load() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return Err(e);
        }
    };

    // Initialize OIDC validator
    let oidc_validator = match OidcValidator::new(&config.oidc).await {
        Ok(validator) => {
            info!("OIDC validator initialized");
            Arc::new(validator)
        }
        Err(e) => {
            eprintln!("Failed to initialize OIDC validator: {}", e);
            return Err(e);
        }
    };

    // Initialize PostgreSQL pool
    let postgres_pool = match PostgresPool::new(&config.database).await {
        Ok(pool) => {
            info!("PostgreSQL connection pool initialized");
            pool
        }
        Err(e) => {
            eprintln!("Failed to initialize PostgreSQL pool: {}", e);
            return Err(e);
        }
    };

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

    let listener = match tokio::net::TcpListener::bind(&config.server.bind_address).await {
        Ok(listener) => {
            info!("Server starting on {}", config.server.bind_address);
            listener
        }
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}", config.server.bind_address, e);
            return Err(e.into());
        }
    };

    info!("Server is ready to accept connections");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
