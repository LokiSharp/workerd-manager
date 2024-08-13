use crate::{config::AppState, errors::AuthError};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, get_current_timestamp, DecodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
}

#[async_trait]
impl<S> FromRequestParts<S> for AccessTokenClaims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

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
    type Rejection = AuthError;

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
    pub current_refresh_token_expires_at: Option<usize>,
    pub exp: usize,
}

async fn decoding_token_from_request_parts<T: serde::de::DeserializeOwned>(
    parts: &mut Parts,
    decoding: DecodingKey,
) -> Result<T, AuthError> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| AuthError::InvalidToken)?;

    let token_data = decode::<T>(bearer.token(), &decoding, &Validation::default())
        .map_err(|_| AuthError::InvalidToken)?;

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

pub fn generate_token_pair(
    state: &AppState,
    user_id: &str,
    current_refresh_token: Option<&str>,
    current_refresh_token_expires_at: Option<usize>,
) -> Result<(String, String), AuthError> {
    // TODO take username from the database
    let access_token = AccessTokenClaims {
        sub: user_id.to_owned(),
        username: "me".to_owned(),
        exp: get_current_timestamp() as usize + 60 * 60,
    };

    Ok((
        encode(
            &Header::default(),
            &access_token,
            &state.jwt_auth_keys.encoding,
        )
        .expect("Failed to encode access token"),
        generate_refresh_token(
            state,
            user_id,
            current_refresh_token,
            current_refresh_token_expires_at,
        )
        .expect("Failed to generate refresh token"),
    ))
}

pub fn generate_refresh_token(
    state: &AppState,
    user_id: &str,
    current_refresh_token: Option<&str>,
    current_refresh_token_expires_at: Option<usize>,
) -> Result<String, AuthError> {
    if current_refresh_token.is_some() && current_refresh_token_expires_at.is_some() {
        if is_refresh_token_black_listed(state, current_refresh_token.unwrap(), user_id) {
            return Err(AuthError::InvalidToken);
        }
        // TODO Put the refresh token in the blacklist
        todo!()
    }

    let refresh_token = RefreshTokenClaims {
        sub: user_id.to_owned(),
        current_refresh_token: current_refresh_token.map(|s| s.to_owned()),
        current_refresh_token_expires_at,
        exp: get_current_timestamp() as usize + 60 * 60 * 24 * 7,
    };

    Ok(encode(
        &Header::default(),
        &refresh_token,
        &state.jwt_refresh_keys.encoding,
    )
    .expect("Failed to encode refresh token"))
}

pub fn is_refresh_token_black_listed(state: &AppState, refresh_token: &str, user_id: &str) -> bool {
    // TODO Check if the refresh token is blacklisted
    todo!()
}
