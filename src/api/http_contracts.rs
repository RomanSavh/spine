use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::db::queries;
use crate::embed::{pipeline, search::f32_to_bytes};
use crate::error::AppError;
use crate::models::{HttpContract, HttpContractSummary};
use crate::AppState;

#[utoipa::path(get, path = "/http-contracts", tag = "HTTP Contracts",
    responses((status = 200, description = "List all HTTP contracts", body = Vec<HttpContractSummary>))
)]
pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<HttpContractSummary>>, AppError> {
    let db = state.db.lock().unwrap();
    Ok(Json(queries::list_http_contracts(&db)?))
}

#[utoipa::path(get, path = "/http-contracts/{service}", tag = "HTTP Contracts",
    params(("service" = String, Path, description = "Service name")),
    responses(
        (status = 200, description = "HTTP contract found", body = HttpContract),
        (status = 404, description = "HTTP contract not found"),
    )
)]
pub async fn get_one(
    State(state): State<AppState>,
    Path(service): Path<String>,
) -> Result<Json<HttpContract>, AppError> {
    let db = state.db.lock().unwrap();
    let h = queries::get_http_contract(&db, &service)?.ok_or(AppError::NotFound)?;
    Ok(Json(h))
}

#[utoipa::path(post, path = "/http-contracts", tag = "HTTP Contracts",
    request_body = HttpContract,
    responses((status = 201, description = "HTTP contract created", body = HttpContract))
)]
pub async fn create(
    State(state): State<AppState>,
    Json(h): Json<HttpContract>,
) -> Result<(StatusCode, Json<HttpContract>), AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::insert_http_contract(&db, &h)?;
    }
    embed_http(&state, &h).await;
    Ok((StatusCode::CREATED, Json(h)))
}

#[utoipa::path(put, path = "/http-contracts/{service}", tag = "HTTP Contracts",
    params(("service" = String, Path, description = "Service name")),
    request_body = HttpContract,
    responses(
        (status = 200, description = "HTTP contract updated", body = HttpContract),
        (status = 404, description = "HTTP contract not found"),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(service): Path<String>,
    Json(h): Json<HttpContract>,
) -> Result<Json<HttpContract>, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::update_http_contract(&db, &service, &h)?;
    }
    embed_http(&state, &h).await;
    Ok(Json(h))
}

#[utoipa::path(delete, path = "/http-contracts/{service}", tag = "HTTP Contracts",
    params(("service" = String, Path, description = "Service name")),
    responses(
        (status = 204, description = "HTTP contract deleted"),
        (status = 404, description = "HTTP contract not found"),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(service): Path<String>,
) -> Result<StatusCode, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::delete_http_contract(&db, &service)?;
    }
    state.embeddings.write().unwrap().remove("http", &service);
    Ok(StatusCode::NO_CONTENT)
}

async fn embed_http(state: &AppState, h: &HttpContract) {
    let text = pipeline::embed_text_for_http(h);
    let hash = pipeline::text_hash(&text);
    {
        let db = state.db.lock().unwrap();
        if let Ok(Some(existing)) = queries::get_embedding_hash(&db, "http", &h.service) {
            if existing == hash {
                return;
            }
        }
    }
    if let Ok(vector) = state.embed_client.embed(&text).await {
        let bytes = f32_to_bytes(&vector);
        let db = state.db.lock().unwrap();
        let _ = queries::upsert_embedding(&db, "http", &h.service, &hash, &bytes);
        drop(db);
        state
            .embeddings
            .write()
            .unwrap()
            .upsert("http", &h.service, vector);
    }
}
