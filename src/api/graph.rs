use axum::extract::{Path, State};
use axum::Json;

use crate::db::queries;
use crate::error::AppError;
use crate::graph;
use crate::AppState;

#[utoipa::path(get, path = "/graph/dependencies/{service}", tag = "Graph",
    params(("service" = String, Path, description = "Service name")),
    responses(
        (status = 200, description = "Dependency graph", body = graph::DependencyGraph),
        (status = 404, description = "Service not found"),
    )
)]
pub async fn dependencies(
    State(state): State<AppState>,
    Path(service): Path<String>,
) -> Result<Json<graph::DependencyGraph>, AppError> {
    let db = state.db.lock().unwrap();
    let all_services = queries::list_services_full(&db)?;
    drop(db);

    graph::compute_dependencies(&service, &all_services)
        .map(Json)
        .ok_or(AppError::NotFound)
}
