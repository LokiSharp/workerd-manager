use crate::{
    auth::{hash_password, AccessTokenClaims},
    config::AppState,
    errors::ServerError,
};

use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use entity::sea_orm_active_enums::RoleEnum;
use service::users::{Mutation, Query};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UserCreateRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UserInfoResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub roles: Vec<RoleEnum>,
    pub status: i32,
}

#[debug_handler]
pub async fn create_user(
    State(state): State<AppState>,
    Json(new_user): Json<UserCreateRequest>,
) -> Result<String, ServerError> {
    let hashed_password = match hash_password(&new_user.password) {
        Ok(hash) => hash,
        Err(err) => {
            tracing::error!("Failed to hash password: {:?}", err);
            return Err(ServerError::InternalServerError);
        }
    };

    Mutation::create_user(
        &state.db,
        new_user.email,
        new_user.username,
        hashed_password,
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
    claims: AccessTokenClaims,
    Path(id): Path<String>,
) -> Result<Json<UserInfoResponse>, ServerError> {
    if claims.sub != id && !claims.roles.contains(&RoleEnum::Admin) {
        tracing::error!("Unauthorized access: {:?}", claims);
        return Err(ServerError::Unauthorized);
    }
    let user = Query::find_user_by_id(&state.db, id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get user: {:?}", err);
            ServerError::InternalServerError
        })?
        .ok_or(ServerError::NotFound)?;

    Ok(Json(UserInfoResponse {
        id: user.id.to_string(),
        email: user.email,
        username: user.username,
        roles: user.roles,
        status: user.status,
    }))
}

#[debug_handler]
pub async fn get_all_users(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
) -> Result<Json<Vec<UserInfoResponse>>, ServerError> {
    if !claims.roles.contains(&RoleEnum::Admin) {
        tracing::error!("Unauthorized access: {:?}", claims);
        return Err(ServerError::Unauthorized);
    }

    let users = Query::find_all_users(&state.db).await.map_err(|err| {
        tracing::error!("Failed to get all users: {:?}", err);
        ServerError::InternalServerError
    })?;

    Ok(Json(
        users
            .into_iter()
            .map(|user| UserInfoResponse {
                id: user.id.to_string(),
                email: user.email,
                username: user.username,
                roles: user.roles,
                status: user.status,
            })
            .collect(),
    ))
}

#[debug_handler]
pub async fn update_user(
    State(state): State<AppState>,
    claims: AccessTokenClaims,
    Path(id): Path<String>,
    Json(user): Json<UserCreateRequest>,
) -> Result<String, ServerError> {
    if claims.sub != id && !claims.roles.contains(&RoleEnum::Admin) {
        tracing::error!("Unauthorized access: {:?}", claims);
        return Err(ServerError::Unauthorized);
    }

    let hashed_password = match hash_password(&user.password) {
        Ok(hash) => hash,
        Err(err) => {
            tracing::error!("Failed to hash password: {:?}", err);
            return Err(ServerError::InternalServerError);
        }
    };

    Mutation::update_user(&state.db, id, user.email, user.username, hashed_password)
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
    claims: AccessTokenClaims,
    Path(id): Path<String>,
) -> Result<String, ServerError> {
    if claims.sub != id && !claims.roles.contains(&RoleEnum::Admin) {
        tracing::error!("Unauthorized access: {:?}", claims);
        return Err(ServerError::Unauthorized);
    }
    Mutation::delete_user(&state.db, id)
        .await
        .map(|_| "User deleted successfully".to_owned())
        .map_err(|err| {
            tracing::error!("Failed to delete user: {:?}", err);
            ServerError::InternalServerError
        })
}
