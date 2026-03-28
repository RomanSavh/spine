use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::embed::search::SearchResult;
use crate::error::AppError;
use crate::AppState;

#[derive(Deserialize, IntoParams)]
pub struct SearchParams {
    /// Natural language search query
    pub q: String,
    /// Filter by entity type: service, table, queue, proto, http
    #[serde(rename = "type")]
    #[param(rename = "type")]
    pub type_filter: Option<String>,
    /// Max results to return (default 20)
    pub limit: Option<usize>,
}

#[utoipa::path(get, path = "/search", tag = "Search",
    params(SearchParams),
    responses((status = 200, description = "Semantic search results", body = Vec<SearchResult>))
)]
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let query_vec = state
        .embed_client
        .embed(&params.q)
        .await
        .map_err(AppError::Embed)?;

    let index = state.embeddings.read().unwrap();
    let results = index.search(
        &query_vec,
        params.type_filter.as_deref(),
        params.limit.unwrap_or(20),
    );
    Ok(Json(results))
}
