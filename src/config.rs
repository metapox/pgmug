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
    pub skip_validation: Option<bool>, // 開発環境用
    pub dev_secret: Option<String>, // 開発環境用のHS256秘密鍵
}

impl Config {
    pub fn load() -> Result<Self> {
        println!("Loading configuration...");
        eprintln!("Loading configuration...");
        
        // Load .env file if it exists
        if let Err(_) = dotenvy::dotenv() {
            println!("No .env file found, using only environment variables");
        } else {
            println!(".env file loaded successfully");
        }
        
        let config = config::Config::builder()
            .add_source(config::File::with_name("config.yaml").required(false))
            .add_source(
                config::Environment::with_prefix("POSTGRES_PROXY")
                    .prefix_separator("_")
                    .separator("__") // 二重アンダースコアを階層セパレーターとして明示的に指定
                    .try_parsing(true) // 数値や真偽値を自動変換
            )
            .build();
            
        println!("Config builder result: {:?}", config.is_ok());
        let config = config?;
        
        println!("Attempting to deserialize config...");
        let config_result = config.try_deserialize::<Config>();
        println!("Deserialize result: {:?}", config_result.is_ok());
        let config = config_result?;
        
        println!("Configuration loaded successfully");
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
                skip_validation: Some(false),
                dev_secret: None,
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
        assert_eq!(config.oidc.skip_validation, None);
        assert_eq!(config.oidc.dev_secret, None);
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
                skip_validation: Some(true),
                dev_secret: Some("test_dev_secret".to_string()),
            },
        };

        assert_eq!(config.server.bind_address, "0.0.0.0:8080");
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.oidc.issuer_url, "https://test.auth0.com");
        assert_eq!(config.oidc.client_id, "test-client");
        assert_eq!(config.oidc.audience, Some("test-audience".to_string()));
        assert_eq!(config.oidc.skip_validation, Some(true));
        assert_eq!(config.oidc.dev_secret, Some("test_dev_secret".to_string()));
    }
}
