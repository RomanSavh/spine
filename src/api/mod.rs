use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::AppState;

mod context;
mod graph;
mod health;
mod http_contracts;
mod protos;
mod queues;
mod search;
mod services;
mod tables;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Spine — System Knowledge Registry",
        description = "A read-only-first registry of system architecture knowledge for AI agents. \
                       Stores services, database tables, queue contracts, proto contracts, and HTTP contracts. \
                       Supports semantic search via embeddings.",
        version = "0.1.0"
    ),
    paths(
        services::list, services::get_one, services::create, services::update, services::delete,
        tables::list, tables::get_one, tables::create, tables::update, tables::delete,
        queues::list, queues::get_one, queues::create, queues::update, queues::delete,
        protos::list, protos::get_one, protos::create, protos::update, protos::delete,
        http_contracts::list, http_contracts::get_one, http_contracts::create, http_contracts::update, http_contracts::delete,
        graph::dependencies,
        search::search,
        context::get_context,
        health::health,
    ),
    components(schemas(
        crate::models::Service, crate::models::ServiceSummary,
        crate::models::Table, crate::models::TableSummary,
        crate::models::QueueContract, crate::models::QueueSchema, crate::models::QueueField, crate::models::QueueSummary,
        crate::models::ProtoContract, crate::models::ProtoSummary,
        crate::models::HttpContract, crate::models::HttpContractSummary,
        crate::graph::DependencyGraph, crate::graph::Dependency,
        crate::embed::search::SearchResult,
        context::ServiceContext,
    )),
    tags(
        (name = "Services", description = "Microservice registry"),
        (name = "Tables", description = "Database table schemas"),
        (name = "Queues", description = "Queue/topic message contracts"),
        (name = "Proto Contracts", description = "gRPC proto definitions"),
        (name = "HTTP Contracts", description = "HTTP/OpenAPI specifications"),
        (name = "Graph", description = "Service dependency graph"),
        (name = "Search", description = "Semantic search across all entities"),
        (name = "Context", description = "Bundled service context"),
        (name = "Health", description = "Service health and stats"),
    )
)]
pub struct ApiDoc;

pub fn router<T>(state: AppState, mcp_service: T) -> Router
where
    T: tower::Service<axum::http::Request<axum::body::Body>, Error = std::convert::Infallible>
        + Clone
        + Send
        + Sync
        + 'static,
    T::Response: axum::response::IntoResponse,
    T::Future: Send,
{
    Router::new()
        // MCP (SSE-based Streamable HTTP)
        .nest_service("/mcp", mcp_service)
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Services
        .route("/services", get(services::list).post(services::create))
        .route(
            "/services/{name}",
            get(services::get_one)
                .put(services::update)
                .delete(services::delete),
        )
        // Tables
        .route("/tables", get(tables::list).post(tables::create))
        .route(
            "/tables/{name}",
            get(tables::get_one)
                .put(tables::update)
                .delete(tables::delete),
        )
        // Queues
        .route("/queues", get(queues::list).post(queues::create))
        .route(
            "/queues/{name}",
            get(queues::get_one)
                .put(queues::update)
                .delete(queues::delete),
        )
        // Protos
        .route("/protos", get(protos::list).post(protos::create))
        .route(
            "/protos/{server}",
            get(protos::get_one)
                .put(protos::update)
                .delete(protos::delete),
        )
        // HTTP Contracts
        .route(
            "/http-contracts",
            get(http_contracts::list).post(http_contracts::create),
        )
        .route(
            "/http-contracts/{service}",
            get(http_contracts::get_one)
                .put(http_contracts::update)
                .delete(http_contracts::delete),
        )
        // Graph
        .route(
            "/graph/dependencies/{service}",
            get(graph::dependencies),
        )
        // Search
        .route("/search", get(search::search))
        // Context
        .route("/context/{service}", get(context::get_context))
        // Health
        .route("/health", get(health::health))
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
