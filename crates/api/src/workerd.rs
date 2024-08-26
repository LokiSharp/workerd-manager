use std::{collections::HashMap, path::PathBuf};

use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use entity::sea_orm_active_enums::RoleEnum;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use service::workers::Query;
use sha2::{Digest, Sha256};
use tokio::{fs, process::Command, sync::oneshot};

use crate::{auth::AccessTokenClaims, config::AppState, errors::ServerError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Worker {
    pub id: String,
    pub host_name: String,
    pub port: String,
    pub entry: String,
    pub code: String,
    pub template: Option<String>,
}

#[debug_handler]
pub async fn write_worker_config_capfile(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let worker = get_worker_with_id(state.to_owned(), claims, id.to_owned())
        .await
        .map_err(|err| {
            tracing::error!("Failed to get worker: {:?}", err);
            ServerError::WorkerNotFound
        })?;

    let file_map = generate_worker_configs(&state, vec![worker.clone()]).await;
    let file_content = file_map.get(&worker.id).unwrap().clone();

    let path = PathBuf::from(state.env.workerd_dir.to_string())
        .join("worker-info")
        .join(worker.id)
        .join("Capfile");

    fs::create_dir_all(path.parent().unwrap())
        .await
        .map_err(|err| {
            tracing::error!("Failed to create directories: {:?}", err);
            ServerError::InternalServerError
        })?;

    fs::write(path, file_content).await.map_err(|err| {
        tracing::error!("Failed to write file: {:?}", err);
        ServerError::InternalServerError
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Capfile written" })),
    ))
}

#[debug_handler]
pub async fn write_worker_code(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let worker = get_worker_with_id(state.to_owned(), claims, id.to_owned())
        .await
        .map_err(|err| {
            tracing::error!("Failed to get worker: {:?}", err);
            ServerError::WorkerNotFound
        })?;

    let path = PathBuf::from(&state.env.workerd_dir.to_string())
        .join(state.env.worker_info_dir.to_string())
        .join(worker.id)
        .join("src")
        .join(worker.entry);

    fs::create_dir_all(path.parent().unwrap())
        .await
        .map_err(|err| {
            tracing::error!("Failed to create directories: {:?}", err);
            ServerError::InternalServerError
        })?;

    fs::write(path, worker.code).await.map_err(|err| {
        tracing::error!("Failed to write file: {:?}", err);
        ServerError::InternalServerError
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Worker code written" })),
    ))
}

#[debug_handler]
pub async fn delete_file(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let worker = get_worker_with_id(state.to_owned(), claims, id.to_owned())
        .await
        .map_err(|err| {
            tracing::error!("Failed to get worker: {:?}", err);
            ServerError::WorkerNotFound
        })?;

    let path = PathBuf::from(state.env.workerd_dir.to_string())
        .join(state.env.worker_info_dir.to_string())
        .join(worker.id);

    fs::remove_dir_all(path).await.map_err(|err| {
        tracing::error!("Failed to delete file: {:?}", err);
        ServerError::InternalServerError
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Worker file deleted" })),
    ))
}

#[debug_handler]
pub async fn run_cmd(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let worker = get_worker_with_id(state.to_owned(), claims, id.to_owned())
        .await
        .map_err(|err| {
            tracing::error!("Failed to get worker: {:?}", err);
            ServerError::WorkerNotFound
        })?;

    let mut chan_map = state.chan_map.lock().await;

    if chan_map.contains_key(&worker.id) {
        tracing::error!("{} is still running!", id);
        return Err(ServerError::WorkerStillRunning);
    }

    let (tx, rx) = oneshot::channel();
    chan_map.insert(worker.id.clone(), tx);

    tokio::spawn(async move {
        let worker_dir = PathBuf::from(state.env.workerd_dir.to_string())
            .join("worker-info")
            .join(worker.id.clone());

        let args = vec![
            "serve".to_string(),
            worker_dir.join("Capfile").to_str().unwrap().to_string(),
            "--watch".to_string(),
            "--verbose".to_string(),
        ]
        .into_iter()
        .collect::<Vec<_>>();

        let child = Command::new(state.env.workerd_bin_path.to_string())
            .args(&args)
            .spawn()
            .map_err(|err| {
                tracing::error!("Failed to start subprocess: {:?}", err);
                ServerError::FailedStartWorker
            })
            .unwrap();

        let mut child_map = state.child_map.lock().await;
        child_map.insert(worker.id.clone(), child);

        let _ = rx.await;
        let mut child = child_map.remove(&worker.id).unwrap();
        let _ = child.kill().await;
    });

    Ok((
        StatusCode::OK,
        Json(json!({ "message": format!("{} is running!", id) })),
    ))
}

#[debug_handler]
pub async fn exit_cmd(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let worker = get_worker_with_id(state.to_owned(), claims, id.to_owned())
        .await
        .map_err(|err| {
            tracing::error!("Failed to get worker: {:?}", err);
            ServerError::WorkerNotFound
        })?;

    let mut chan_map = state.chan_map.lock().await;

    if let Some(tx) = chan_map.remove(&worker.id) {
        let _ = tx.send(());
    } else {
        return Err(ServerError::WorkerNotRunning);
    }

    Ok((
        StatusCode::OK,
        Json(json!({ "message": format!("{} exited", id) })),
    ))
}

#[debug_handler]
pub async fn exit_all_cmd(State(state): State<AppState>) -> Result<impl IntoResponse, ServerError> {
    let mut chan_map = state.chan_map.lock().await;

    for (_, tx) in chan_map.drain() {
        let _ = tx.send(());
    }

    Ok((StatusCode::OK, "All commands exited").into_response())
}

fn get_template_hash(template: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(template);
    format!("{:x}", hasher.finalize())
}

async fn generate_worker_configs(
    state: &AppState,
    workers: Vec<Worker>,
) -> HashMap<String, String> {
    let mut results = HashMap::new();
    for worker in workers {
        let config = generate_worker_config(state, worker.clone()).await;
        results.insert(worker.id.clone(), config);
    }
    results
}

async fn generate_worker_config(state: &AppState, mut worker: Worker) -> String {
    worker.id = worker.id.replace("-", "");
    let template = worker
        .clone()
        .template
        .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());
    let template_hash = get_template_hash(&template);

    let mut template_cache = state.template_cache.lock().await;
    let compiled_template = template_cache
        .entry(template_hash.clone())
        .or_insert_with(|| {
            let mut handlebars = Handlebars::new();
            handlebars
                .register_template_string(&template_hash, &template)
                .unwrap();
            handlebars
        });

    compiled_template.render(&template_hash, &worker).unwrap()
}

pub async fn get_worker_with_id(
    state: AppState,
    claims: AccessTokenClaims,
    id: String,
) -> Result<Worker, ServerError> {
    let worker_in_db = Query::find_worker_by_id(&state.db, id.clone())
        .await
        .map_err(|err| {
            tracing::error!("Failed to get worker: {:?}", err);
            ServerError::InternalServerError
        })?
        .ok_or(ServerError::NotFound)?;

    if claims.sub != worker_in_db.user_id.to_string() && !claims.roles.contains(&RoleEnum::Admin) {
        tracing::error!("Unauthorized access: {:?}", claims);
        return Err(ServerError::Unauthorized);
    }

    let mut worker = Worker {
        id,
        host_name: worker_in_db.host_name.to_string(),
        port: worker_in_db.port.to_string(),
        entry: worker_in_db.entry.to_string(),
        code: worker_in_db.code.to_string(),
        template: worker_in_db.template.map(|id| id.to_string()),
    };

    worker.id = worker.id.replace("-", "");
    Ok(worker)
}

const DEFAULT_TEMPLATE: &str = r#"using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "{{id}}", worker = .worker{{id}}),
  ],

  sockets = [
    (
      name = "{{id}}",
      address = "{{host_name}}:{{port}}",
      http = (),
      service = "{{id}}"
    ),
  ]
);

const worker{{id}} :Workerd.Worker = (
  serviceWorkerScript = embed "src/{{entry}}",
  compatibilityDate = "2024-06-03",
);
"#;
