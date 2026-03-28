
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct HttpContract {
    pub service: String,
    pub description: String,
    pub spec_raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HttpContractSummary {
    pub service: String,
    pub description: String,
}
