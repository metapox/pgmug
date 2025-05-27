use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_postgres::{Client, NoTls};
use tracing::{info, warn};

use crate::config::DatabaseConfig;

#[derive(Clone)]
pub struct PostgresPool {
    config: DatabaseConfig,
    semaphore: Arc<Semaphore>,
}

impl PostgresPool {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        // Test connection
        let connection_string = format!(
            "host={} port={} user={} password={} dbname={}",
            config.host, config.port, config.username, config.password, config.database
        );

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;

        // Spawn the connection in the background
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                warn!("Database connection error: {}", e);
            }
        });

        // Test the connection
        client.simple_query("SELECT 1").await?;
        info!("Database connection test successful");

        Ok(Self {
            config: config.clone(),
            semaphore: Arc::new(Semaphore::new(config.max_connections as usize)),
        })
    }

    pub async fn get_client(&self) -> Result<PostgresClient> {
        // Acquire a permit from the semaphore
        let permit = self.semaphore.clone().acquire_owned().await?;

        // Create a new connection
        let connection_string = format!(
            "host={} port={} user={} password={} dbname={}",
            self.config.host,
            self.config.port,
            self.config.username,
            self.config.password,
            self.config.database
        );

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;

        // Spawn the connection in the background
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                warn!("Database connection error: {}", e);
            }
        });

        Ok(PostgresClient {
            client,
            _permit: permit,
        })
    }
}

pub struct PostgresClient {
    client: Client,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl std::ops::Deref for PostgresClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
