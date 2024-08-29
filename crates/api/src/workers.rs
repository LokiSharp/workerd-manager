use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use entity::{sea_orm_active_enums::RoleEnum, worker};
use service::workers::{Mutation, Query};

use crate::{auth::AccessTokenClaims, config::AppState, errors::ServerError};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WorkerCreateRequest {
    pub name: String,
    pub port: i32,
    pub code: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WorkerUpdateRequest {
    pub external_path: Option<String>,
    pub host_name: Option<String>,
    pub node_name: Option<String>,
    pub port: Option<i32>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub tunnel_id: Option<String>,
    pub template: Option<String>,
    pub user_id: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WorkerInfoResponse {
    pub id: String,
    pub external_path: String,
    pub host_name: String,
    pub node_name: String,
    pub port: i32,
    pub code: String,
    pub name: String,
    pub tunnel_id: Option<String>,
    pub template: Option<String>,
    pub user_id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[debug_handler]
pub async fn create_worker(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Json(worker): Json<WorkerCreateRequest>,
) -> Result<Json<MessageResponse>, ServerError> {
    Mutation::create_worker(
        &state.db,
        worker.name,
        worker.port,
        worker.code,
        claims.sub.clone(),
    )
    .await
    .map(|_| {
        Json(MessageResponse {
            message: "Worker created successfully".to_owned(),
        })
    })
    .map_err(|err| {
        tracing::error!("Failed to create worker: {:?}", err);
        ServerError::InternalServerError
    })
}

#[debug_handler]
pub async fn get_worker(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Path(id): Path<String>,
) -> Result<Json<WorkerInfoResponse>, ServerError> {
    let worker = Query::find_worker_by_id(&state.db, id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get worker: {:?}", err);
            ServerError::InternalServerError
        })?
        .ok_or(ServerError::NotFound)?;

    if claims.sub != worker.user_id.to_string() && !claims.roles.contains(&RoleEnum::Admin) {
        tracing::error!("Unauthorized access: {:?}", claims);
        return Err(ServerError::Unauthorized);
    }

    Ok(Json(WorkerInfoResponse {
        id: worker.id.to_string(),
        external_path: worker.external_path,
        host_name: worker.host_name,
        node_name: worker.node_name,
        port: worker.port,
        code: worker.code,
        name: worker.name,
        tunnel_id: worker.tunnel_id.map(|id| id.to_string()),
        template: worker.template.map(|id| id.to_string()),
        user_id: worker.user_id.to_string(),
    }))
}

#[debug_handler]
pub async fn get_all_workers(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
) -> Result<Json<Vec<WorkerInfoResponse>>, ServerError> {
    let workers: Vec<worker::Model>;

    if claims.roles.contains(&RoleEnum::Admin) {
        workers = Query::find_all_workers(&state.db).await.map_err(|err| {
            tracing::error!("Failed to get all workers: {:?}", err);
            ServerError::InternalServerError
        })?;
    } else {
        workers = Query::find_user_workers_with_user_id(&state.db, claims.sub)
            .await
            .map_err(|err| {
                tracing::error!("Failed to get all workers: {:?}", err);
                ServerError::InternalServerError
            })?;
    }

    Ok(Json(
        workers
            .into_iter()
            .map(|worker| WorkerInfoResponse {
                id: worker.id.to_string(),
                external_path: worker.external_path,
                host_name: worker.host_name,
                node_name: worker.node_name,
                port: worker.port,
                code: worker.code,
                name: worker.name,
                tunnel_id: worker.tunnel_id.map(|id| id.to_string()),
                template: worker.template.map(|id| id.to_string()),
                user_id: worker.user_id.to_string(),
            })
            .collect(),
    ))
}

#[debug_handler]
pub async fn update_worker(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Path(id): Path<String>,
    Json(worker_request): Json<WorkerUpdateRequest>,
) -> Result<Json<MessageResponse>, ServerError> {
    let worker = Query::find_worker_by_id(&state.db, id.to_owned())
        .await
        .map_err(|err| {
            tracing::error!("Failed to get worker: {:?}", err);
            ServerError::InternalServerError
        })?
        .ok_or(ServerError::NotFound)?;

    if claims.sub != worker.user_id.to_string() && !claims.roles.contains(&RoleEnum::Admin) {
        tracing::error!("Unauthorized access: {:?}", claims);
        return Err(ServerError::Unauthorized);
    }

    Mutation::update_worker(
        &state.db,
        id,
        worker_request.external_path.unwrap_or(worker.external_path),
        worker_request.host_name.unwrap_or(worker.host_name),
        worker_request.node_name.unwrap_or(worker.node_name),
        worker_request.port.unwrap_or(worker.port),
        worker_request.code.unwrap_or(worker.code),
        worker_request.name.unwrap_or(worker.name),
        worker_request.tunnel_id,
        worker_request.template,
    )
    .await
    .map(|_| {
        Json(MessageResponse {
            message: "Worker updated successfully".to_owned(),
        })
    })
    .map_err(|err| {
        tracing::error!("Failed to update worker: {:?}", err);
        ServerError::InternalServerError
    })
}

#[debug_handler]
pub async fn delete_worker(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Path(id): Path<String>,
) -> Result<Json<MessageResponse>, ServerError> {
    let worker = Query::find_worker_by_id(&state.db, id.to_owned())
        .await
        .map_err(|err| {
            tracing::error!("Failed to get worker: {:?}", err);
            ServerError::InternalServerError
        })?
        .ok_or(ServerError::NotFound)?;

    if claims.sub != worker.user_id.to_string() && !claims.roles.contains(&RoleEnum::Admin) {
        tracing::error!("Unauthorized access: {:?}", claims);
        return Err(ServerError::Unauthorized);
    }

    Mutation::delete_worker(&state.db, id)
        .await
        .map(|_| {
            Json(MessageResponse {
                message: "Worker deleted successfully".to_owned(),
            })
        })
        .map_err(|err| {
            tracing::error!("Failed to delete worker: {:?}", err);
            ServerError::InternalServerError
        })
}
