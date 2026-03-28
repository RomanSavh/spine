use std::sync::{Arc, Mutex, RwLock};

mod api;
mod config;
mod db;
mod embed;
mod error;
mod graph;
mod mcp;
mod models;

use embed::client::EmbedClient;
use embed::search::EmbeddingIndex;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub embeddings: Arc<RwLock<EmbeddingIndex>>,
    pub embed_client: EmbedClient,
    pub config: config::AppConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "spine=info,tower_http=info".into()),
        )
        .init();

    let cfg = config::AppConfig::from_env();

    let conn = rusqlite::Connection::open(&cfg.db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
    db::schema::migrate(&conn)?;
    tracing::info!("database initialized at {}", cfg.db_path);

    // Load existing embeddings into memory
    let embedding_rows = db::queries::load_all_embeddings(&conn)?;
    let rows: Vec<(String, String, Vec<u8>)> = embedding_rows
        .into_iter()
        .map(|r| (r.entity_type, r.entity_key, r.embedding))
        .collect();
    let index = EmbeddingIndex::load_from_rows(rows);
    tracing::info!("loaded embedding index");

    let db = Arc::new(Mutex::new(conn));
    let embeddings = Arc::new(RwLock::new(index));
    let embed_client = EmbedClient::new(&cfg.embed_service_url);

    let state = AppState {
        db: db.clone(),
        embeddings: embeddings.clone(),
        embed_client: embed_client.clone(),
        config: cfg.clone(),
    };

    // Build MCP HTTP service
    let mcp_service = {
        use rmcp::transport::streamable_http_server::{
            session::local::LocalSessionManager, StreamableHttpServerConfig,
            StreamableHttpService,
        };

        let session_manager = Arc::new(LocalSessionManager::default());
        let config = StreamableHttpServerConfig::default();

        StreamableHttpService::new(
            move || Ok(mcp::SpineMcp::new(db.clone(), embeddings.clone(), embed_client.clone())),
            session_manager,
            config,
        )
    };

    // Build router: REST API + MCP on same server
    let app = api::router(state, mcp_service);

    let addr = format!("0.0.0.0:{}", cfg.port);
    tracing::info!("listening on {addr}");
    tracing::info!("swagger UI at http://localhost:{}/swagger-ui/", cfg.port);
    tracing::info!("MCP endpoint at http://localhost:{}/mcp", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
