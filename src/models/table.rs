
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Table {
    pub name: String,
    pub database: String,
    pub owner: String,
    pub description: String,
    pub ddl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TableSummary {
    pub name: String,
    pub description: String,
}
