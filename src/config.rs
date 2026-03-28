use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub port: u16,
    pub db_path: String,
    pub embed_service_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            port: env::var("SPINE_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            db_path: env::var("SPINE_DB_PATH").unwrap_or_else(|_| "spine.db".to_string()),
            embed_service_url: env::var("SPINE_EMBED_URL")
                .unwrap_or_else(|_| "http://localhost:8100".to_string()),
        }
    }
}
