use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::db::queries;
use crate::embed::{pipeline, search::f32_to_bytes};
use crate::error::AppError;
use crate::models::{Service, ServiceSummary};
use crate::AppState;

#[utoipa::path(get, path = "/services", tag = "Services", operation_id = "list_services",
    responses((status = 200, description = "List all services", body = Vec<ServiceSummary>))
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<ServiceSummary>>, AppError> {
    let db = state.db.lock().unwrap();
    let services = queries::list_services(&db)?;
    Ok(Json(services))
}

#[utoipa::path(get, path = "/services/{name}", tag = "Services", operation_id = "get_service",
    params(("name" = String, Path, description = "Service name")),
    responses(
        (status = 200, description = "Service found", body = Service),
        (status = 404, description = "Service not found"),
    )
)]
pub async fn get_one(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Service>, AppError> {
    let db = state.db.lock().unwrap();
    let svc = queries::get_service(&db, &name)?.ok_or(AppError::NotFound)?;
    Ok(Json(svc))
}

#[utoipa::path(post, path = "/services", tag = "Services", operation_id = "create_service",
    request_body = Service,
    responses((status = 201, description = "Service created", body = Service))
)]
pub async fn create(
    State(state): State<AppState>,
    Json(svc): Json<Service>,
) -> Result<(StatusCode, Json<Service>), AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::insert_service(&db, &svc)?;
    }
    embed_service(&state, &svc).await;
    Ok((StatusCode::CREATED, Json(svc)))
}

#[utoipa::path(put, path = "/services/{name}", tag = "Services", operation_id = "update_service",
    params(("name" = String, Path, description = "Service name")),
    request_body = Service,
    responses(
        (status = 200, description = "Service updated", body = Service),
        (status = 404, description = "Service not found"),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(svc): Json<Service>,
) -> Result<Json<Service>, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::update_service(&db, &name, &svc)?;
    }
    embed_service(&state, &svc).await;
    Ok(Json(svc))
}

#[utoipa::path(delete, path = "/services/{name}", tag = "Services", operation_id = "delete_service",
    params(("name" = String, Path, description = "Service name")),
    responses(
        (status = 204, description = "Service deleted"),
        (status = 404, description = "Service not found"),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    {
        let db = state.db.lock().unwrap();
        queries::delete_service(&db, &name)?;
    }
    state
        .embeddings
        .write()
        .unwrap()
        .remove("service", &name);
    Ok(StatusCode::NO_CONTENT)
}

async fn embed_service(state: &AppState, svc: &Service) {
    let text = pipeline::embed_text_for_service(svc);
    let hash = pipeline::text_hash(&text);

    {
        let db = state.db.lock().unwrap();
        if let Ok(Some(existing)) = queries::get_embedding_hash(&db, "service", &svc.name) {
            if existing == hash {
                return;
            }
        }
    }

    if let Ok(vector) = state.embed_client.embed(&text).await {
        let bytes = f32_to_bytes(&vector);
        let db = state.db.lock().unwrap();
        let _ = queries::upsert_embedding(&db, "service", &svc.name, &hash, &bytes);
        drop(db);
        state
            .embeddings
            .write()
            .unwrap()
            .upsert("service", &svc.name, vector);
    }
}
