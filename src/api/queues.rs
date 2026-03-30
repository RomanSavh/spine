use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::db::queries;
use crate::embed::{pipeline, search::f32_to_bytes};
use crate::error::AppError;
use crate::models::{QueueContract, QueueSummary};
use crate::AppState;

#[utoipa::path(get, path = "/queues", tag = "Queues", operation_id = "list_queues",
    responses((status = 200, description = "List all queue contracts", body = Vec<QueueSummary>))
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<QueueSummary>>, AppError> {
    let db = state.db.lock().unwrap();
    Ok(Json(queries::list_queues(&db)?))
}

#[utoipa::path(get, path = "/queues/{name}", tag = "Queues", operation_id = "get_queue",
    params(("name" = String, Path, description = "Queue name")),
    responses(
        (status = 200, description = "Queue contract found", body = QueueContract),
        (status = 404, description = "Queue not found"),
    )
)]
pub async fn get_one(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<QueueContract>, AppError> {
    let db = state.db.lock().unwrap();
    let q = queries::get_queue(&db, &name)?.ok_or(AppError::NotFound)?;
    Ok(Json(q))
}

#[utoipa::path(post, path = "/queues", tag = "Queues", operation_id = "create_queue",
    request_body = QueueContract,
    responses((status = 201, description = "Queue contract created", body = QueueContract))
)]
pub async fn create(
    State(state): State<AppState>,
    Json(q): Json<QueueContract>,
) -> Result<(StatusCode, Json<QueueContract>), AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::insert_queue(&db, &q)?;
    }
    embed_queue(&state, &q).await;
    Ok((StatusCode::CREATED, Json(q)))
}

#[utoipa::path(put, path = "/queues/{name}", tag = "Queues", operation_id = "update_queue",
    params(("name" = String, Path, description = "Queue name")),
    request_body = QueueContract,
    responses(
        (status = 200, description = "Queue contract updated", body = QueueContract),
        (status = 404, description = "Queue not found"),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(q): Json<QueueContract>,
) -> Result<Json<QueueContract>, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::update_queue(&db, &name, &q)?;
    }
    embed_queue(&state, &q).await;
    Ok(Json(q))
}

#[utoipa::path(delete, path = "/queues/{name}", tag = "Queues", operation_id = "delete_queue",
    params(("name" = String, Path, description = "Queue name")),
    responses(
        (status = 204, description = "Queue contract deleted"),
        (status = 404, description = "Queue not found"),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::delete_queue(&db, &name)?;
    }
    state.embeddings.write().unwrap().remove("queue", &name);
    Ok(StatusCode::NO_CONTENT)
}

async fn embed_queue(state: &AppState, q: &QueueContract) {
    let text = pipeline::embed_text_for_queue(q);
    let hash = pipeline::text_hash(&text);
    {
        let db = state.db.lock().unwrap();
        if let Ok(Some(existing)) = queries::get_embedding_hash(&db, "queue", &q.name) {
            if existing == hash {
                return;
            }
        }
    }
    if let Ok(vector) = state.embed_client.embed(&text).await {
        let bytes = f32_to_bytes(&vector);
        let db = state.db.lock().unwrap();
        let _ = queries::upsert_embedding(&db, "queue", &q.name, &hash, &bytes);
        drop(db);
        state
            .embeddings
            .write()
            .unwrap()
            .upsert("queue", &q.name, vector);
    }
}
