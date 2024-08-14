use crate::{config::AppState, errors::ServerError};

use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use service::Mutation;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UserInfo {
    pub email: String,
    pub username: String,
}

#[debug_handler]
pub async fn create_user(
    State(state): State<AppState>,
    Json(new_user): Json<CreateUserRequest>,
) -> Result<String, ServerError> {
    Mutation::create_user(
        &state.db,
        new_user.email,
        new_user.username,
        new_user.password,
    )
    .await
    .map(|_| "User created successfully".to_owned())
    .map_err(|err| {
        tracing::error!("Failed to create user: {:?}", err);
        ServerError::InternalServerError
    })
}

#[debug_handler]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserInfo>, ServerError> {
    let user = service::Query::find_user_by_id(&state.db, id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get user: {:?}", err);
            ServerError::InternalServerError
        })?
        .ok_or(ServerError::NotFound)?;

    Ok(Json(UserInfo {
        email: user.email,
        username: user.username,
    }))
}

#[debug_handler]
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(user): Json<CreateUserRequest>,
) -> Result<String, ServerError> {
    Mutation::update_user(&state.db, id, user.email, user.username, user.password)
        .await
        .map(|_| "User updated successfully".to_owned())
        .map_err(|err| {
            tracing::error!("Failed to update user: {:?}", err);
            ServerError::InternalServerError
        })
}

#[debug_handler]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<String, ServerError> {
    Mutation::delete_user(&state.db, id)
        .await
        .map(|_| "User deleted successfully".to_owned())
        .map_err(|err| {
            tracing::error!("Failed to delete user: {:?}", err);
            ServerError::InternalServerError
        })
}
