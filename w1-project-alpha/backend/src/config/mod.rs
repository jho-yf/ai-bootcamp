use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    #[allow(dead_code)] // Reserved for future use with connection pool configuration
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        
        let config = Config {
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .map_err(|_| "DATABASE_URL must be set")?,
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
            },
            server: ServerConfig {
                host: std::env::var("HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()?,
            },
        };
        
        Ok(config)
    }
}
