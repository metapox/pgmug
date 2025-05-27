use anyhow::{anyhow, Result};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::config::OidcConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: Option<serde_json::Value>,
    pub exp: usize,
    pub iat: usize,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwksKey {
    kty: String,
    #[serde(rename = "use")]
    key_use: Option<String>,
    kid: Option<String>,
    n: Option<String>,
    e: Option<String>,
    x: Option<String>,
    y: Option<String>,
    crv: Option<String>,
    alg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Jwks {
    keys: Vec<JwksKey>,
}

struct CachedJwks {
    jwks: Jwks,
    cached_at: Instant,
}

pub struct OidcValidator {
    config: OidcConfig,
    client: reqwest::Client,
    jwks_cache: Arc<RwLock<Option<CachedJwks>>>,
}

impl OidcValidator {
    pub async fn new(config: &OidcConfig) -> Result<Self> {
        let client = reqwest::Client::new();
        
        let validator = Self {
            config: config.clone(),
            client,
            jwks_cache: Arc::new(RwLock::new(None)),
        };

        // Pre-load JWKS
        validator.fetch_jwks().await?;
        info!("OIDC validator initialized with issuer: {}", config.issuer_url);

        Ok(validator)
    }

    async fn fetch_jwks(&self) -> Result<Jwks> {
        let jwks_url = format!("{}/.well-known/jwks.json", self.config.issuer_url);
        let response = self.client.get(&jwks_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch JWKS: HTTP {}", response.status()));
        }

        let jwks: Jwks = response.json().await?;
        
        // Cache the JWKS
        {
            let mut cache = self.jwks_cache.write().await;
            *cache = Some(CachedJwks {
                jwks: jwks.clone(),
                cached_at: Instant::now(),
            });
        }

        Ok(jwks)
    }

    async fn get_jwks(&self) -> Result<Jwks> {
        {
            let cache = self.jwks_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                let cache_age = cached.cached_at.elapsed();
                if cache_age < Duration::from_secs(self.config.jwks_cache_duration_seconds) {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        // Cache expired or doesn't exist, fetch new JWKS
        self.fetch_jwks().await
    }

    fn find_key<'a>(&self, jwks: &'a Jwks, kid: Option<&str>) -> Option<&'a JwksKey> {
        if let Some(kid) = kid {
            jwks.keys.iter().find(|key| key.kid.as_deref() == Some(kid))
        } else {
            jwks.keys.first()
        }
    }

    fn create_decoding_key(&self, key: &JwksKey) -> Result<DecodingKey> {
        match key.kty.as_str() {
            "RSA" => {
                let n = key.n.as_ref().ok_or_else(|| anyhow!("Missing 'n' parameter for RSA key"))?;
                let e = key.e.as_ref().ok_or_else(|| anyhow!("Missing 'e' parameter for RSA key"))?;
                
                DecodingKey::from_rsa_components(n, e).map_err(|e| anyhow!("Failed to create RSA key: {}", e))
            }
            "EC" => {
                let x = key.x.as_ref().ok_or_else(|| anyhow!("Missing 'x' parameter for EC key"))?;
                let y = key.y.as_ref().ok_or_else(|| anyhow!("Missing 'y' parameter for EC key"))?;
                
                DecodingKey::from_ec_components(x, y).map_err(|e| anyhow!("Failed to create EC key: {}", e))
            }
            _ => Err(anyhow!("Unsupported key type: {}", key.kty)),
        }
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims> {
        // Decode header to get kid
        let header = decode_header(token)?;
        let kid = header.kid.as_deref();

        // Get JWKS
        let jwks = self.get_jwks().await?;

        // Find the appropriate key
        let key = self.find_key(&jwks, kid)
            .ok_or_else(|| anyhow!("No matching key found for token"))?;

        // Create decoding key
        let decoding_key = self.create_decoding_key(key)?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.config.issuer_url]);
        
        if let Some(audience) = &self.config.audience {
            validation.set_audience(&[audience]);
        } else if !self.config.client_id.is_empty() {
            validation.set_audience(&[&self.config.client_id]);
        }

        // Decode and validate token
        let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
        
        Ok(token_data.claims)
    }
}

pub async fn auth_middleware(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication for health check
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    // Extract Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Extract bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            warn!("Invalid Authorization header format");
            StatusCode::UNAUTHORIZED
        })?;

    // Validate token
    match state.oidc_validator.validate_token(token).await {
        Ok(claims) => {
            info!("User authenticated: {}", claims.sub);
            // Add user info to request extensions if needed
            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!("Token validation failed: {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
