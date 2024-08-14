use crate::errors::ConfigError;
use jsonwebtoken::{DecodingKey, EncodingKey};
use service::sea_orm::{Database, DatabaseConnection};
use std::borrow::Cow;
use tracing::error;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis_client: redis::Client,
    pub env: EnvironmentVariables,
    pub jwt_auth_keys: Keys,
    pub jwt_refresh_keys: Keys,
}

impl AppState {
    pub async fn from_env() -> Result<Self, ConfigError> {
        let env = EnvironmentVariables::from_env()?;
        Ok(Self {
            db: Database::connect(&*env.database_url).await.map_err(|err| {
                error!("failed to connect to the database: {:?}", err);
                ConfigError::FailedDatabaseConnection
            })?,
            redis_client: redis::Client::open(env.redis_url.as_ref()).map_err(|err| {
                error!("failed to connect to Redis: {:?}", err);
                ConfigError::FailedRedisConnection
            })?,
            env: env.clone(),
            jwt_auth_keys: Keys::new(env.clone().jwt_secret.as_bytes()),
            jwt_refresh_keys: Keys::new(env.clone().jwt_refresh_secret.as_bytes()),
        })
    }
}

#[derive(Clone, Debug)]
pub struct EnvironmentVariables {
    pub api_listen_addr: Cow<'static, str>,
    pub api_port: u16,
    pub database_type: Cow<'static, str>,
    pub database_url: Cow<'static, str>,
    pub redis_url: Cow<'static, str>,
    pub jwt_secret: Cow<'static, str>,
    pub jwt_refresh_secret: Cow<'static, str>,
}

impl EnvironmentVariables {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();

        fn get_env_var(key: &str) -> Result<String, ConfigError> {
            match dotenv::var(key) {
                Ok(value) => Ok(value),
                Err(err) => {
                    error!("missing {key}: {err}");
                    Err(ConfigError::FailedReadEnvironment)
                }
            }
        }

        Ok(Self {
            api_listen_addr: get_env_var("API_LISTEN_ADDR")?.into(),
            api_port: match dotenv::var("API_PORT") {
                Ok(s) => match s.parse::<u16>() {
                    Ok(port) => port,
                    Err(_) => return Err(ConfigError::FailedParseEnvironment),
                },
                _ => 8000,
            },
            database_type: get_env_var("DATABASE_TYPE")?.into(),
            database_url: get_env_var("DATABASE_URL")?.into(),
            redis_url: get_env_var("REDIS_URL")?.into(),
            jwt_secret: get_env_var("JWT_SECRET")?.into(),
            jwt_refresh_secret: get_env_var("JWT_REFRESH_SECRET")?.into(),
        })
    }
}

#[derive(Clone)]
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
