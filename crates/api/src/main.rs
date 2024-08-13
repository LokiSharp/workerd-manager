pub mod auth;
pub mod config;
pub mod errors;

use crate::auth::AuthBody;
use crate::config::AppState;
use crate::errors::AuthError;
use auth::{generate_token_pair, AuthPayload, RefreshTokenClaims};
use axum::extract::State;
use axum::{
    routing::{get, post},
    Json, Router,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
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

async fn index() -> Result<String, AuthError> {
    Ok(format!("Hello, World!",))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AuthError> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    // TODO Check the credentials against the database
    if payload.email != "me@example.com" || payload.password != "password" {
        return Err(AuthError::WrongCredentials);
    }

    let (access_token, refresh_token) =
        generate_token_pair(&state, "1", None, None).expect("Failed to generate token pair");

    Ok(Json(AuthBody::new(access_token, refresh_token)))
}

async fn refresh_token(
    State(state): State<AppState>,
    claims: RefreshTokenClaims,
) -> Result<Json<AuthBody>, AuthError> {
    if claims.sub.is_empty() {
        return Err(AuthError::InvalidToken);
    }

    let (access_token, refresh_token) = generate_token_pair(
        &state,
        claims.sub.as_str(),
        claims.current_refresh_token.as_deref(),
        claims.current_refresh_token_expires_at,
    )
    .expect("Failed to generate token pair");

    Ok(Json(AuthBody::new(access_token, refresh_token)))
}
