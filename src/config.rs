use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub oidc: OidcConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub bind_address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub audience: Option<String>,
    pub jwks_cache_duration_seconds: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config.yaml").required(false))
            .add_source(config::Environment::with_prefix("POSTGRES_PROXY"))
            .build()?;

        let config: Config = config.try_deserialize()?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                bind_address: "0.0.0.0:8080".to_string(),
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "password".to_string(),
                database: "postgres".to_string(),
                max_connections: 10,
            },
            oidc: OidcConfig {
                issuer_url: "https://your-oidc-provider.com".to_string(),
                client_id: "your-client-id".to_string(),
                audience: None,
                jwks_cache_duration_seconds: 3600,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.server.bind_address, "0.0.0.0:8080");
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.database.username, "postgres");
        assert_eq!(config.database.password, "password");
        assert_eq!(config.database.database, "postgres");
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.oidc.issuer_url, "https://your-oidc-provider.com");
        assert_eq!(config.oidc.client_id, "your-client-id");
        assert_eq!(config.oidc.audience, None);
        assert_eq!(config.oidc.jwks_cache_duration_seconds, 3600);
    }

    #[test]
    fn test_config_structure() {
        let config = Config {
            server: ServerConfig {
                bind_address: "0.0.0.0:8080".to_string(),
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "password".to_string(),
                database: "postgres".to_string(),
                max_connections: 10,
            },
            oidc: OidcConfig {
                issuer_url: "https://test.auth0.com".to_string(),
                client_id: "test-client".to_string(),
                audience: Some("test-audience".to_string()),
                jwks_cache_duration_seconds: 3600,
            },
        };

        assert_eq!(config.server.bind_address, "0.0.0.0:8080");
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.oidc.issuer_url, "https://test.auth0.com");
        assert_eq!(config.oidc.client_id, "test-client");
        assert_eq!(config.oidc.audience, Some("test-audience".to_string()));
    }
}
