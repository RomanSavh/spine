use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::db::queries;
use crate::error::AppError;
use crate::models::*;
use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct ServiceContext {
    pub service: Service,
    pub tables: Vec<Table>,
    pub queue_contracts: Vec<QueueContract>,
    pub proto_contracts: Vec<ProtoContract>,
    pub http_contracts: Vec<HttpContract>,
}

#[utoipa::path(get, path = "/context/{service}", tag = "Context",
    params(("service" = String, Path, description = "Service name")),
    responses(
        (status = 200, description = "Bundled service context", body = ServiceContext),
        (status = 404, description = "Service not found"),
    )
)]
pub async fn get_context(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ServiceContext>, AppError> {
    let db = state.db.lock().unwrap();

    let service = queries::get_service(&db, &name)?.ok_or(AppError::NotFound)?;

    let tables = queries::get_tables_by_names(&db, &service.tables)?;

    let mut queue_names: Vec<String> = service.queue_publishers.clone();
    queue_names.extend(service.queue_subscribers.clone());
    queue_names.sort();
    queue_names.dedup();
    let queue_contracts = queries::get_queues_by_names(&db, &queue_names)?;

    let mut grpc_names: Vec<String> = service.grpc_servers.clone();
    grpc_names.extend(service.grpc_clients.clone());
    grpc_names.sort();
    grpc_names.dedup();
    let proto_contracts = queries::get_protos_by_servers(&db, &grpc_names)?;

    let http_contracts = if service.http_server {
        queries::get_http_contract(&db, &service.name)?
            .into_iter()
            .collect()
    } else {
        vec![]
    };

    Ok(Json(ServiceContext {
        service,
        tables,
        queue_contracts,
        proto_contracts,
        http_contracts,
    }))
}
