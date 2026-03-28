
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ProtoContract {
    pub server: String,
    pub description: String,
    pub proto_raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProtoSummary {
    pub server: String,
    pub description: String,
}
