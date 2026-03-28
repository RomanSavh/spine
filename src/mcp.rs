use std::sync::{Arc, Mutex, RwLock};

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, ErrorData, InitializeResult, ServerCapabilities};
use rmcp::ServerHandler;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::db::queries;
use crate::embed::client::EmbedClient;
use crate::embed::search::EmbeddingIndex;

#[derive(Clone)]
pub struct SpineMcp {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub embeddings: Arc<RwLock<EmbeddingIndex>>,
    pub embed_client: EmbedClient,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl SpineMcp {
    pub fn new(
        db: Arc<Mutex<rusqlite::Connection>>,
        embeddings: Arc<RwLock<EmbeddingIndex>>,
        embed_client: EmbedClient,
    ) -> Self {
        Self {
            db,
            embeddings,
            embed_client,
            tool_router: Self::tool_router(),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NameParam {
    /// Name of the entity to look up
    name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ServerParam {
    /// gRPC server name
    server: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ServiceParam {
    /// Service name
    service: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchParam {
    /// Natural language search query
    query: String,
    /// Filter by entity type: service, table, queue, proto, http
    entity_type: Option<String>,
    /// Max results (default 10)
    limit: Option<usize>,
}

#[rmcp::tool_router]
impl SpineMcp {
    #[rmcp::tool(description = "List all registered services (names and descriptions)")]
    fn list_services(&self) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        let services = queries::list_services(&db)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&services).unwrap();
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[rmcp::tool(description = "Get full details of a service by name, including gRPC servers/clients, queue publishers/subscribers, and owned tables")]
    fn get_service(
        &self,
        Parameters(params): Parameters<NameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        match queries::get_service(&db, &params.name) {
            Ok(Some(svc)) => {
                let json = serde_json::to_string_pretty(&svc).unwrap();
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Ok(None) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Service '{}' not found",
                params.name
            ))])),
            Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
        }
    }

    #[rmcp::tool(description = "List all registered database tables (names and descriptions)")]
    fn list_tables(&self) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        let tables = queries::list_tables(&db)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&tables).unwrap();
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[rmcp::tool(description = "Get full details of a database table by name, including DDL schema")]
    fn get_table(
        &self,
        Parameters(params): Parameters<NameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        match queries::get_table(&db, &params.name) {
            Ok(Some(t)) => {
                let json = serde_json::to_string_pretty(&t).unwrap();
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Ok(None) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Table '{}' not found",
                params.name
            ))])),
            Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
        }
    }

    #[rmcp::tool(description = "List all registered queue contracts (names and descriptions)")]
    fn list_queues(&self) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        let queues = queries::list_queues(&db)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&queues).unwrap();
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[rmcp::tool(description = "Get full details of a queue contract by name, including message schema")]
    fn get_queue(
        &self,
        Parameters(params): Parameters<NameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        match queries::get_queue(&db, &params.name) {
            Ok(Some(q)) => {
                let json = serde_json::to_string_pretty(&q).unwrap();
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Ok(None) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Queue '{}' not found",
                params.name
            ))])),
            Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
        }
    }

    #[rmcp::tool(description = "Get proto contract by gRPC server name, including raw .proto definition")]
    fn get_proto(
        &self,
        Parameters(params): Parameters<ServerParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        match queries::get_proto(&db, &params.server) {
            Ok(Some(p)) => {
                let json = serde_json::to_string_pretty(&p).unwrap();
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Ok(None) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Proto contract for server '{}' not found",
                params.server
            ))])),
            Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
        }
    }

    #[rmcp::tool(description = "Get HTTP contract by service name, including OpenAPI spec")]
    fn get_http_contract(
        &self,
        Parameters(params): Parameters<ServiceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        match queries::get_http_contract(&db, &params.service) {
            Ok(Some(h)) => {
                let json = serde_json::to_string_pretty(&h).unwrap();
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Ok(None) => Ok(CallToolResult::error(vec![Content::text(format!(
                "HTTP contract for service '{}' not found",
                params.service
            ))])),
            Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
        }
    }

    #[rmcp::tool(description = "Get full context for a service: the service plus all its tables, queue contracts, proto contracts, and HTTP contracts")]
    fn get_context(
        &self,
        Parameters(params): Parameters<ServiceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        let svc = match queries::get_service(&db, &params.service) {
            Ok(Some(s)) => s,
            Ok(None) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Service '{}' not found",
                    params.service
                ))]));
            }
            Err(e) => return Err(ErrorData::internal_error(e.to_string(), None)),
        };

        let tables = queries::get_tables_by_names(&db, &svc.tables)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let mut queue_names: Vec<String> = svc.queue_publishers.clone();
        queue_names.extend(svc.queue_subscribers.clone());
        queue_names.sort();
        queue_names.dedup();
        let queue_contracts = queries::get_queues_by_names(&db, &queue_names)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let proto_contracts = queries::get_protos_by_servers(&db, &svc.grpc_servers)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let http_contracts: Vec<crate::models::HttpContract> = if svc.http_server {
            queries::get_http_contract(&db, &svc.name)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
                .into_iter()
                .collect()
        } else {
            vec![]
        };

        let context = serde_json::json!({
            "service": svc,
            "tables": tables,
            "queue_contracts": queue_contracts,
            "proto_contracts": proto_contracts,
            "http_contracts": http_contracts,
        });
        let json = serde_json::to_string_pretty(&context).unwrap();
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[rmcp::tool(description = "Get the dependency graph for a service: which services it depends on and which depend on it")]
    fn get_dependencies(
        &self,
        Parameters(params): Parameters<ServiceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        let all_services = queries::list_services_full(&db)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        drop(db);

        match crate::graph::compute_dependencies(&params.service, &all_services) {
            Some(graph) => {
                let json = serde_json::to_string_pretty(&graph).unwrap();
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "Service '{}' not found",
                params.service
            ))])),
        }
    }

    #[rmcp::tool(description = "Semantic search across all entities using natural language. Returns results ranked by relevance.")]
    async fn search(
        &self,
        Parameters(params): Parameters<SearchParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let query_vec = self
            .embed_client
            .embed(&params.query)
            .await
            .map_err(|e| ErrorData::internal_error(e, None))?;

        let index = self.embeddings.read().unwrap();
        let results = index.search(
            &query_vec,
            params.entity_type.as_deref(),
            params.limit.unwrap_or(10),
        );
        let json = serde_json::to_string_pretty(&results).unwrap();
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

impl ServerHandler for SpineMcp {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "Spine is a system knowledge registry. Use it to understand microservice architecture, \
                 database schemas, queue contracts, gRPC/HTTP APIs, and service dependencies."
            )
    }
}
