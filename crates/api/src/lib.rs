pub mod auth;
pub mod config;
pub mod errors;
pub mod users;
pub mod workerd;
pub mod workers;

use crate::config::AppState;
use crate::errors::ServerError;
use auth::{login, refresh_token};
use axum::{
    http::{self, Method},
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use users::{create_user, delete_user, get_all_users, get_user, update_user};
use workerd::{delete_file, exit_cmd, run_cmd, write_worker_code, write_worker_config_capfile};
use workers::{create_worker, delete_worker, get_all_workers, get_worker, update_worker};

#[tokio::main]
pub async fn start() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_origin(Any)
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
            http::header::AUTHORIZATION,
        ]);

    let state = AppState::from_env()
        .await
        .expect("Failed to load configuration");

    let app = Router::new()
        .route("/", get(index))
        .route("/auth/login", post(login))
        .route("/auth/refresh-tokens", post(refresh_token))
        .route("/users", get(get_all_users).post(create_user))
        .route(
            "/users/:id",
            get(get_user).patch(update_user).delete(delete_user),
        )
        .route("/workers", get(get_all_workers).post(create_worker))
        .route(
            "/workers/:id",
            get(get_worker).patch(update_worker).delete(delete_worker),
        )
        .route("/workers/:id/config", post(write_worker_config_capfile))
        .route("/workers/:id/code", post(write_worker_code))
        .route("/workers/:id/file", delete(delete_file))
        .route("/workers/:id/exec", post(run_cmd).delete(exit_cmd))
        .layer(cors)
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        state.env.api_listen_addr, state.env.api_port
    ))
    .await
    .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Result<String, ServerError> {
    Ok(format!("Hello, World!",))
}
