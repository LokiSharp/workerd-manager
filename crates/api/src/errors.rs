use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ConfigError {
    FailedReadEnvironment,
    FailedParseEnvironment,
    FailedDatabaseConnection,
    FailedRedisConnection,
}

#[derive(Debug)]
pub enum ServerError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InternalServerError,
    FailedToEncodeAccessToken,
    FailedToDecodeAccessToken,
    FailedToEncodeRefreshToken,
    FailedToDecodeRefreshToken,
    FailedToGenerateTokenPair,
    NotFound,
    Unauthorized,
    MissingAuthorizationHeader,
    InvalidAuthorizationHeader,
    WorkerStillRunning,
    WorkerNotRunning,
    WorkerNotFound,
    FailedStartWorker,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ServerError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            ServerError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            ServerError::TokenCreation => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error")
            }
            ServerError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            ServerError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ServerError::FailedToEncodeAccessToken => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to encode access token",
            ),
            ServerError::FailedToDecodeAccessToken => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to decode access token",
            ),
            ServerError::FailedToEncodeRefreshToken => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to encode refresh token",
            ),
            ServerError::FailedToDecodeRefreshToken => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to decode refresh token",
            ),
            ServerError::FailedToGenerateTokenPair => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate token pair",
            ),
            ServerError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            ServerError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ServerError::MissingAuthorizationHeader => {
                (StatusCode::BAD_REQUEST, "Missing authorization header")
            }
            ServerError::InvalidAuthorizationHeader => {
                (StatusCode::BAD_REQUEST, "Invalid authorization header")
            }
            ServerError::WorkerStillRunning => (StatusCode::BAD_REQUEST, "Worker is still running"),
            ServerError::WorkerNotRunning => (StatusCode::BAD_REQUEST, "Worker is not running"),
            ServerError::WorkerNotFound => (StatusCode::NOT_FOUND, "Worker not found"),
            ServerError::FailedStartWorker => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to start worker")
            }
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
