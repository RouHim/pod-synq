use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub db_path: String,
    pub admin_username: Option<String>,
    pub admin_password: Option<String>,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let port = env::var("PODSYNQ_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let db_path = env::var("PODSYNQ_DB_PATH").unwrap_or_else(|_| "./pod-synq.db".to_string());

        let admin_username = env::var("PODSYNQ_ADMIN_USERNAME").ok();
        let admin_password = env::var("PODSYNQ_ADMIN_PASSWORD").ok();

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            port,
            db_path,
            admin_username,
            admin_password,
            log_level,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port must be between 1 and 65535".to_string());
        }

        if self.db_path.is_empty() {
            return Err("Database path cannot be empty".to_string());
        }

        Ok(())
    }
}
