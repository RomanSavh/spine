
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct QueueField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct QueueSchema {
    pub name: String,
    pub fields: Vec<QueueField>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct QueueContract {
    pub name: String,
    pub description: String,
    pub schema: QueueSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QueueSummary {
    pub name: String,
    pub description: String,
}
