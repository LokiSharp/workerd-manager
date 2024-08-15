use crate::{config::AppState, errors::ServerError};
use axum::{
    async_trait, debug_handler,
    extract::{FromRef, FromRequestParts, State},
    http::request::Parts,
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use entity::sea_orm_active_enums::RoleEnum;
use jsonwebtoken::{
    decode, encode, get_current_timestamp, DecodingKey, Header, TokenData, Validation,
};
use redis::Commands;
use serde::{Deserialize, Serialize};
use service::Query;
use std::fmt::Display;

#[debug_handler]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, ServerError> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(ServerError::MissingCredentials);
    }

    let user = Query::find_user_by_email(&state.db, payload.email.clone())
        .await
        .map_err(|_| ServerError::InternalServerError)?
        .ok_or(ServerError::WrongCredentials)?;
    if payload.email != user.email || payload.password != user.password {
        return Err(ServerError::WrongCredentials);
    }

    let (access_token, refresh_token) =
        generate_token_pair(&state, &user.id.to_string(), None, None)
            .await
            .map_err(|_| ServerError::FailedToGenerateTokenPair)?;

    Ok(Json(AuthBody::new(access_token, refresh_token)))
}

#[debug_handler]
pub async fn refresh_token(
    State(state): State<AppState>,
    claims: RefreshTokenClaims,
) -> Result<Json<AuthBody>, ServerError> {
    if claims.sub.is_empty() {
        return Err(ServerError::InvalidToken);
    }

    let (access_token, refresh_token) = generate_token_pair(
        &state,
        &claims.sub,
        claims.current_refresh_token.as_deref(),
        claims.current_refresh_token_expires_at,
    )
    .await
    .map_err(|_| ServerError::FailedToGenerateTokenPair)?;

    Ok(Json(AuthBody::new(access_token, refresh_token)))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub username: String,
    pub roles: Vec<RoleEnum>,
    pub status: i32,
    pub exp: u64,
}

#[async_trait]
impl<S> FromRequestParts<S> for AccessTokenClaims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let claims = decoding_token_from_request_parts::<AccessTokenClaims>(
            parts,
            state.jwt_refresh_keys.decoding,
        )
        .await?;

        Ok(claims)
    }
}

impl Display for AccessTokenClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {}\nUsername: {}", self.sub, self.username)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RefreshTokenClaims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let claims = decoding_token_from_request_parts::<RefreshTokenClaims>(
            parts,
            state.jwt_refresh_keys.decoding,
        )
        .await?;

        Ok(claims)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,
    pub current_refresh_token: Option<String>,
    pub current_refresh_token_expires_at: Option<u64>,
    pub exp: u64,
}

async fn decoding_token_from_request_parts<T: serde::de::DeserializeOwned>(
    parts: &mut Parts,
    decoding: DecodingKey,
) -> Result<T, ServerError> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| ServerError::InvalidToken)?;

    let token_data = decode::<T>(bearer.token(), &decoding, &Validation::default())
        .map_err(|_| ServerError::InvalidToken)?;

    Ok(token_data.claims)
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
}

pub async fn generate_token_pair(
    state: &AppState,
    user_id: &str,
    current_refresh_token: Option<&str>,
    current_refresh_token_expires_at: Option<u64>,
) -> Result<(String, String), ServerError> {
    let user = Query::find_user_by_id(&state.db, user_id.to_string())
        .await
        .map_err(|_| ServerError::InternalServerError)?
        .ok_or(ServerError::WrongCredentials)?;

    let access_token = AccessTokenClaims {
        sub: user.id.to_string().to_owned(),
        username: user.username.to_owned(),
        roles: user.roles,
        status: user.status,
        exp: get_current_timestamp() + 60 * 60,
    };

    Ok((
        encode(
            &Header::default(),
            &access_token,
            &state.jwt_auth_keys.encoding,
        )
        .map_err(|_| ServerError::FailedToEncodeAccessToken)?,
        generate_refresh_token(
            state,
            user_id,
            current_refresh_token,
            current_refresh_token_expires_at,
        )
        .map_err(|_| ServerError::FailedToEncodeRefreshToken)?,
    ))
}

pub fn generate_refresh_token(
    state: &AppState,
    user_id: &str,
    current_refresh_token: Option<&str>,
    current_refresh_token_expires_at: Option<u64>,
) -> Result<String, ServerError> {
    if current_refresh_token.is_some() && current_refresh_token_expires_at.is_some() {
        if is_refresh_token_black_listed(state, current_refresh_token.clone().unwrap(), user_id)
            .unwrap()
        {
            return Err(ServerError::InvalidToken);
        }
        blacklist_token(state, current_refresh_token.clone().unwrap(), user_id)
            .expect("Failed to blacklist refresh token");
    }

    let refresh_token = RefreshTokenClaims {
        sub: user_id.to_owned(),
        current_refresh_token: current_refresh_token.map(|s| s.to_owned()),
        current_refresh_token_expires_at,
        exp: get_current_timestamp() + 60 * 60 * 24 * 7,
    };

    Ok(encode(
        &Header::default(),
        &refresh_token,
        &state.jwt_refresh_keys.encoding,
    )
    .map_err(|_| ServerError::FailedToEncodeRefreshToken)?)
}

fn blacklist_token(state: &AppState, token: &str, user_id: &str) -> redis::RedisResult<()> {
    let redis_client = state.redis_client.clone();
    let mut con = redis_client.get_connection()?;

    let token_data: TokenData<RefreshTokenClaims> = decode::<RefreshTokenClaims>(
        &token,
        &state.jwt_refresh_keys.decoding,
        &Validation::default(),
    )
    .expect("Failed to decode refresh token");

    let exp = token_data.claims.exp;
    let current_time = get_current_timestamp();
    let ttl = if exp > current_time {
        exp - current_time
    } else {
        60
    };

    con.set_ex(token, user_id, ttl.try_into().unwrap())
}

pub fn is_refresh_token_black_listed(
    state: &AppState,
    refresh_token: &str,
    user_id: &str,
) -> Result<bool, redis::RedisError> {
    let redis_client = state.redis_client.clone();
    let mut con = redis_client
        .get_connection()
        .expect("Failed to connect to Redis");
    let result: Option<String> = con
        .get(&refresh_token)
        .expect("Failed to get refresh token from Redis");
    Ok(result.map(|s| s == user_id).unwrap_or(false))
}
