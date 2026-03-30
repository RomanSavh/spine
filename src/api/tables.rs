use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::db::queries;
use crate::embed::{pipeline, search::f32_to_bytes};
use crate::error::AppError;
use crate::models::{Table, TableSummary};
use crate::AppState;

#[utoipa::path(get, path = "/tables", tag = "Tables", operation_id = "list_tables",
    responses((status = 200, description = "List all tables", body = Vec<TableSummary>))
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TableSummary>>, AppError> {
    let db = state.db.lock().unwrap();
    Ok(Json(queries::list_tables(&db)?))
}

#[utoipa::path(get, path = "/tables/{name}", tag = "Tables", operation_id = "get_table",
    params(("name" = String, Path, description = "Table name")),
    responses(
        (status = 200, description = "Table found", body = Table),
        (status = 404, description = "Table not found"),
    )
)]
pub async fn get_one(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Table>, AppError> {
    let db = state.db.lock().unwrap();
    let t = queries::get_table(&db, &name)?.ok_or(AppError::NotFound)?;
    Ok(Json(t))
}

#[utoipa::path(post, path = "/tables", tag = "Tables", operation_id = "create_table",
    request_body = Table,
    responses((status = 201, description = "Table created", body = Table))
)]
pub async fn create(
    State(state): State<AppState>,
    Json(t): Json<Table>,
) -> Result<(StatusCode, Json<Table>), AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::insert_table(&db, &t)?;
    }
    embed_table(&state, &t).await;
    Ok((StatusCode::CREATED, Json(t)))
}

#[utoipa::path(put, path = "/tables/{name}", tag = "Tables", operation_id = "update_table",
    params(("name" = String, Path, description = "Table name")),
    request_body = Table,
    responses(
        (status = 200, description = "Table updated", body = Table),
        (status = 404, description = "Table not found"),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(t): Json<Table>,
) -> Result<Json<Table>, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::update_table(&db, &name, &t)?;
    }
    embed_table(&state, &t).await;
    Ok(Json(t))
}

#[utoipa::path(delete, path = "/tables/{name}", tag = "Tables", operation_id = "delete_table",
    params(("name" = String, Path, description = "Table name")),
    responses(
        (status = 204, description = "Table deleted"),
        (status = 404, description = "Table not found"),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::delete_table(&db, &name)?;
    }
    state.embeddings.write().unwrap().remove("table", &name);
    Ok(StatusCode::NO_CONTENT)
}

async fn embed_table(state: &AppState, t: &Table) {
    let text = pipeline::embed_text_for_table(t);
    let hash = pipeline::text_hash(&text);
    {
        let db = state.db.lock().unwrap();
        if let Ok(Some(existing)) = queries::get_embedding_hash(&db, "table", &t.name) {
            if existing == hash {
                return;
            }
        }
    }
    if let Ok(vector) = state.embed_client.embed(&text).await {
        let bytes = f32_to_bytes(&vector);
        let db = state.db.lock().unwrap();
        let _ = queries::upsert_embedding(&db, "table", &t.name, &hash, &bytes);
        drop(db);
        state
            .embeddings
            .write()
            .unwrap()
            .upsert("table", &t.name, vector);
    }
}
