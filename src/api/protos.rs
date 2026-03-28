use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::db::queries;
use crate::embed::{pipeline, search::f32_to_bytes};
use crate::error::AppError;
use crate::models::{ProtoContract, ProtoSummary};
use crate::AppState;

#[utoipa::path(get, path = "/protos", tag = "Proto Contracts",
    responses((status = 200, description = "List all proto contracts", body = Vec<ProtoSummary>))
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<ProtoSummary>>, AppError> {
    let db = state.db.lock().unwrap();
    Ok(Json(queries::list_protos(&db)?))
}

#[utoipa::path(get, path = "/protos/{server}", tag = "Proto Contracts",
    params(("server" = String, Path, description = "gRPC server name")),
    responses(
        (status = 200, description = "Proto contract found", body = ProtoContract),
        (status = 404, description = "Proto contract not found"),
    )
)]
pub async fn get_one(
    State(state): State<AppState>,
    Path(server): Path<String>,
) -> Result<Json<ProtoContract>, AppError> {
    let db = state.db.lock().unwrap();
    let p = queries::get_proto(&db, &server)?.ok_or(AppError::NotFound)?;
    Ok(Json(p))
}

#[utoipa::path(post, path = "/protos", tag = "Proto Contracts",
    request_body = ProtoContract,
    responses((status = 201, description = "Proto contract created", body = ProtoContract))
)]
pub async fn create(
    State(state): State<AppState>,
    Json(p): Json<ProtoContract>,
) -> Result<(StatusCode, Json<ProtoContract>), AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::insert_proto(&db, &p)?;
    }
    embed_proto(&state, &p).await;
    Ok((StatusCode::CREATED, Json(p)))
}

#[utoipa::path(put, path = "/protos/{server}", tag = "Proto Contracts",
    params(("server" = String, Path, description = "gRPC server name")),
    request_body = ProtoContract,
    responses(
        (status = 200, description = "Proto contract updated", body = ProtoContract),
        (status = 404, description = "Proto contract not found"),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(server): Path<String>,
    Json(p): Json<ProtoContract>,
) -> Result<Json<ProtoContract>, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::update_proto(&db, &server, &p)?;
    }
    embed_proto(&state, &p).await;
    Ok(Json(p))
}

#[utoipa::path(delete, path = "/protos/{server}", tag = "Proto Contracts",
    params(("server" = String, Path, description = "gRPC server name")),
    responses(
        (status = 204, description = "Proto contract deleted"),
        (status = 404, description = "Proto contract not found"),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(server): Path<String>,
) -> Result<StatusCode, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::delete_proto(&db, &server)?;
    }
    state.embeddings.write().unwrap().remove("proto", &server);
    Ok(StatusCode::NO_CONTENT)
}

async fn embed_proto(state: &AppState, p: &ProtoContract) {
    let text = pipeline::embed_text_for_proto(p);
    let hash = pipeline::text_hash(&text);
    {
        let db = state.db.lock().unwrap();
        if let Ok(Some(existing)) = queries::get_embedding_hash(&db, "proto", &p.server) {
            if existing == hash {
                return;
            }
        }
    }
    if let Ok(vector) = state.embed_client.embed(&text).await {
        let bytes = f32_to_bytes(&vector);
        let db = state.db.lock().unwrap();
        let _ = queries::upsert_embedding(&db, "proto", &p.server, &hash, &bytes);
        drop(db);
        state
            .embeddings
            .write()
            .unwrap()
            .upsert("proto", &p.server, vector);
    }
}
