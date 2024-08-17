pub mod auth;
pub mod config;
pub mod errors;
pub mod users;
pub mod workers;

use crate::config::AppState;
use crate::errors::ServerError;
use auth::{login, refresh_token};
use axum::{
    routing::{get, post},
    Router,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use users::{create_user, delete_user, get_all_users, get_user, update_user};
use workers::{create_worker, delete_worker, get_all_workers, get_worker, update_worker};

#[tokio::main]
pub async fn start() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

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
