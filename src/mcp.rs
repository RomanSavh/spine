use std::sync::{Arc, Mutex, RwLock};

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, ErrorData, InitializeResult, ServerCapabilities};
use rmcp::ServerHandler;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::db::queries;
use crate::embed::client::EmbedClient;
use crate::embed::search::{f32_to_bytes, EmbeddingIndex};
use crate::embed::pipeline;
use crate::models;

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

    async fn embed_entity(&self, entity_type: &str, entity_key: &str, text: &str) {
        let hash = pipeline::text_hash(text);
        {
            let db = self.db.lock().unwrap();
            if let Ok(Some(existing)) = queries::get_embedding_hash(&db, entity_type, entity_key) {
                if existing == hash {
                    return;
                }
            }
        }
        if let Ok(vector) = self.embed_client.embed(text).await {
            let bytes = f32_to_bytes(&vector);
            let db = self.db.lock().unwrap();
            let _ = queries::upsert_embedding(&db, entity_type, entity_key, &hash, &bytes);
            drop(db);
            self.embeddings
                .write()
                .unwrap()
                .upsert(entity_type, entity_key, vector);
        }
    }

    fn remove_embedding(&self, entity_type: &str, entity_key: &str) {
        self.embeddings
            .write()
            .unwrap()
            .remove(entity_type, entity_key);
    }
}

// ── Read param structs ──

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

// ── Write param structs ──

#[derive(Debug, Deserialize, JsonSchema)]
struct RegisterServiceParam {
    /// Unique service name
    name: String,
    /// What the service does
    description: String,
    /// Link to the source repository
    github_repo: Option<String>,
    /// gRPC servers this service implements
    #[serde(default)]
    grpc_servers: Vec<String>,
    /// gRPC servers this service calls
    #[serde(default)]
    grpc_clients: Vec<String>,
    /// Whether it exposes an HTTP API
    #[serde(default)]
    http_server: bool,
    /// Services it makes HTTP calls to
    #[serde(default)]
    http_clients: Vec<String>,
    /// Queues/topics this service publishes to
    #[serde(default)]
    queue_publishers: Vec<String>,
    /// Queues/topics this service subscribes to
    #[serde(default)]
    queue_subscribers: Vec<String>,
    /// Database tables this service owns
    #[serde(default)]
    tables: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RegisterTableParam {
    /// Table name
    name: String,
    /// Which database it belongs to
    database: String,
    /// Which service owns this table
    owner: String,
    /// What data it holds
    description: String,
    /// Raw PostgreSQL CREATE TABLE statement
    ddl: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct QueueFieldParam {
    /// Field name
    name: String,
    /// Field type (e.g. uuid, string, datetime)
    #[serde(rename = "type")]
    field_type: String,
    /// Field description
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct QueueSchemaParam {
    /// Schema/event name
    name: String,
    /// Typed fields
    fields: Vec<QueueFieldParam>,
    /// Additional notes
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RegisterQueueParam {
    /// Queue/topic name
    name: String,
    /// What events flow through it
    description: String,
    /// Message schema
    schema: QueueSchemaParam,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RegisterProtoParam {
    /// gRPC server name
    server: String,
    /// What the service does
    description: String,
    /// Raw .proto file content
    proto_raw: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RegisterHttpContractParam {
    /// Service name
    service: String,
    /// What the API does
    description: String,
    /// Raw OpenAPI/Swagger spec content
    spec_raw: String,
}

// ── Conversions ──

impl From<RegisterServiceParam> for models::Service {
    fn from(p: RegisterServiceParam) -> Self {
        Self {
            name: p.name,
            description: p.description,
            github_repo: p.github_repo,
            grpc_servers: p.grpc_servers,
            grpc_clients: p.grpc_clients,
            http_server: p.http_server,
            http_clients: p.http_clients,
            queue_publishers: p.queue_publishers,
            queue_subscribers: p.queue_subscribers,
            tables: p.tables,
        }
    }
}

impl From<RegisterTableParam> for models::Table {
    fn from(p: RegisterTableParam) -> Self {
        Self {
            name: p.name,
            database: p.database,
            owner: p.owner,
            description: p.description,
            ddl: p.ddl,
        }
    }
}

impl From<RegisterQueueParam> for models::QueueContract {
    fn from(p: RegisterQueueParam) -> Self {
        Self {
            name: p.name,
            description: p.description,
            schema: models::QueueSchema {
                name: p.schema.name,
                fields: p
                    .schema
                    .fields
                    .into_iter()
                    .map(|f| models::QueueField {
                        name: f.name,
                        field_type: f.field_type,
                        description: f.description,
                    })
                    .collect(),
                notes: p.schema.notes,
            },
        }
    }
}

impl From<RegisterProtoParam> for models::ProtoContract {
    fn from(p: RegisterProtoParam) -> Self {
        Self {
            server: p.server,
            description: p.description,
            proto_raw: p.proto_raw,
        }
    }
}

impl From<RegisterHttpContractParam> for models::HttpContract {
    fn from(p: RegisterHttpContractParam) -> Self {
        Self {
            service: p.service,
            description: p.description,
            spec_raw: p.spec_raw,
        }
    }
}

#[rmcp::tool_router]
impl SpineMcp {
    // ── Read tools ──

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

        let http_contracts: Vec<models::HttpContract> = if svc.http_server {
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

    // ── Write tools ──

    #[rmcp::tool(description = "Register a new service. Provide name, description, and relationship fields (gRPC servers/clients, queues, tables).")]
    async fn register_service(
        &self,
        Parameters(params): Parameters<RegisterServiceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let svc: models::Service = params.into();
        {
            let db = self.db.lock().unwrap();
            queries::insert_service(&db, &svc)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_service(&svc);
        self.embed_entity("service", &svc.name, &text).await;
        let json = serde_json::to_string_pretty(&svc).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Service '{}' registered.\n{json}",
            svc.name
        ))]))
    }

    #[rmcp::tool(description = "Update an existing service. All fields are replaced.")]
    async fn update_service(
        &self,
        Parameters(params): Parameters<RegisterServiceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let svc: models::Service = params.into();
        let name = svc.name.clone();
        {
            let db = self.db.lock().unwrap();
            queries::update_service(&db, &name, &svc)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_service(&svc);
        self.embed_entity("service", &name, &text).await;
        let json = serde_json::to_string_pretty(&svc).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Service '{name}' updated.\n{json}"
        ))]))
    }

    #[rmcp::tool(description = "Delete a service by name.")]
    fn delete_service(
        &self,
        Parameters(params): Parameters<NameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        queries::delete_service(&db, &params.name)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        drop(db);
        self.remove_embedding("service", &params.name);
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Service '{}' deleted.",
            params.name
        ))]))
    }

    #[rmcp::tool(description = "Register a new database table with its DDL schema.")]
    async fn register_table(
        &self,
        Parameters(params): Parameters<RegisterTableParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let t: models::Table = params.into();
        {
            let db = self.db.lock().unwrap();
            queries::insert_table(&db, &t)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_table(&t);
        self.embed_entity("table", &t.name, &text).await;
        let json = serde_json::to_string_pretty(&t).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Table '{}' registered.\n{json}",
            t.name
        ))]))
    }

    #[rmcp::tool(description = "Update an existing database table. All fields are replaced.")]
    async fn update_table(
        &self,
        Parameters(params): Parameters<RegisterTableParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let t: models::Table = params.into();
        let name = t.name.clone();
        {
            let db = self.db.lock().unwrap();
            queries::update_table(&db, &name, &t)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_table(&t);
        self.embed_entity("table", &name, &text).await;
        let json = serde_json::to_string_pretty(&t).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Table '{name}' updated.\n{json}"
        ))]))
    }

    #[rmcp::tool(description = "Delete a database table by name.")]
    fn delete_table(
        &self,
        Parameters(params): Parameters<NameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        queries::delete_table(&db, &params.name)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        drop(db);
        self.remove_embedding("table", &params.name);
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Table '{}' deleted.",
            params.name
        ))]))
    }

    #[rmcp::tool(description = "Register a new queue contract with its message schema.")]
    async fn register_queue(
        &self,
        Parameters(params): Parameters<RegisterQueueParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let q: models::QueueContract = params.into();
        {
            let db = self.db.lock().unwrap();
            queries::insert_queue(&db, &q)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_queue(&q);
        self.embed_entity("queue", &q.name, &text).await;
        let json = serde_json::to_string_pretty(&q).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Queue '{}' registered.\n{json}",
            q.name
        ))]))
    }

    #[rmcp::tool(description = "Update an existing queue contract. All fields are replaced.")]
    async fn update_queue(
        &self,
        Parameters(params): Parameters<RegisterQueueParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let q: models::QueueContract = params.into();
        let name = q.name.clone();
        {
            let db = self.db.lock().unwrap();
            queries::update_queue(&db, &name, &q)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_queue(&q);
        self.embed_entity("queue", &name, &text).await;
        let json = serde_json::to_string_pretty(&q).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Queue '{name}' updated.\n{json}"
        ))]))
    }

    #[rmcp::tool(description = "Delete a queue contract by name.")]
    fn delete_queue(
        &self,
        Parameters(params): Parameters<NameParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        queries::delete_queue(&db, &params.name)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        drop(db);
        self.remove_embedding("queue", &params.name);
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Queue '{}' deleted.",
            params.name
        ))]))
    }

    #[rmcp::tool(description = "Register a new proto contract with raw .proto content.")]
    async fn register_proto(
        &self,
        Parameters(params): Parameters<RegisterProtoParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let p: models::ProtoContract = params.into();
        {
            let db = self.db.lock().unwrap();
            queries::insert_proto(&db, &p)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_proto(&p);
        self.embed_entity("proto", &p.server, &text).await;
        let json = serde_json::to_string_pretty(&p).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Proto '{}' registered.\n{json}",
            p.server
        ))]))
    }

    #[rmcp::tool(description = "Update an existing proto contract. All fields are replaced.")]
    async fn update_proto(
        &self,
        Parameters(params): Parameters<RegisterProtoParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let p: models::ProtoContract = params.into();
        let server = p.server.clone();
        {
            let db = self.db.lock().unwrap();
            queries::update_proto(&db, &server, &p)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_proto(&p);
        self.embed_entity("proto", &server, &text).await;
        let json = serde_json::to_string_pretty(&p).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Proto '{server}' updated.\n{json}"
        ))]))
    }

    #[rmcp::tool(description = "Delete a proto contract by gRPC server name.")]
    fn delete_proto(
        &self,
        Parameters(params): Parameters<ServerParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        queries::delete_proto(&db, &params.server)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        drop(db);
        self.remove_embedding("proto", &params.server);
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Proto '{}' deleted.",
            params.server
        ))]))
    }

    #[rmcp::tool(description = "Register a new HTTP contract with OpenAPI/Swagger spec.")]
    async fn register_http_contract(
        &self,
        Parameters(params): Parameters<RegisterHttpContractParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let h: models::HttpContract = params.into();
        {
            let db = self.db.lock().unwrap();
            queries::insert_http_contract(&db, &h)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_http(&h);
        self.embed_entity("http", &h.service, &text).await;
        let json = serde_json::to_string_pretty(&h).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "HTTP contract '{}' registered.\n{json}",
            h.service
        ))]))
    }

    #[rmcp::tool(description = "Update an existing HTTP contract. All fields are replaced.")]
    async fn update_http_contract(
        &self,
        Parameters(params): Parameters<RegisterHttpContractParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let h: models::HttpContract = params.into();
        let service = h.service.clone();
        {
            let db = self.db.lock().unwrap();
            queries::update_http_contract(&db, &service, &h)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        let text = pipeline::embed_text_for_http(&h);
        self.embed_entity("http", &service, &text).await;
        let json = serde_json::to_string_pretty(&h).unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "HTTP contract '{service}' updated.\n{json}"
        ))]))
    }

    #[rmcp::tool(description = "Delete an HTTP contract by service name.")]
    fn delete_http_contract(
        &self,
        Parameters(params): Parameters<ServiceParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let db = self.db.lock().unwrap();
        queries::delete_http_contract(&db, &params.service)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        drop(db);
        self.remove_embedding("http", &params.service);
        Ok(CallToolResult::success(vec![Content::text(format!(
            "HTTP contract '{}' deleted.",
            params.service
        ))]))
    }
}

impl ServerHandler for SpineMcp {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "Spine is a system knowledge registry. Use it to understand and register microservice architecture, \
                 database schemas, queue contracts, gRPC/HTTP APIs, and service dependencies."
            )
    }
}
