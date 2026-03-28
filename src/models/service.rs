use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Service {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_repo: Option<String>,
    #[serde(default)]
    pub grpc_servers: Vec<String>,
    #[serde(default)]
    pub grpc_clients: Vec<String>,
    #[serde(default)]
    pub http_server: bool,
    #[serde(default)]
    pub http_clients: Vec<String>,
    #[serde(default)]
    pub queue_publishers: Vec<String>,
    #[serde(default)]
    pub queue_subscribers: Vec<String>,
    #[serde(default)]
    pub tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceSummary {
    pub name: String,
    pub description: String,
}
