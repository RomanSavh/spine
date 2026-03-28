use axum::extract::State;
use axum::Json;

use crate::db::queries;
use crate::error::AppError;
use crate::AppState;

#[utoipa::path(get, path = "/health", tag = "Health",
    responses((status = 200, description = "Health status with entity counts"))
)]
pub async fn health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = state.db.lock().unwrap();
    let counts = queries::entity_counts(&db)?;
    let embeddings = queries::embedding_counts(&db)?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "counts": counts,
        "embeddings": embeddings,
    })))
}
