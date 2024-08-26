use crate::errors::ConfigError;
use handlebars::Handlebars;
use jsonwebtoken::{DecodingKey, EncodingKey};
use service::sea_orm::{Database, DatabaseConnection};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tokio::sync::{oneshot, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub redis_client: Arc<redis::Client>,
    pub env: EnvironmentVariables,
    pub jwt_auth_keys: Keys,
    pub jwt_refresh_keys: Keys,
    pub template_cache: Arc<Mutex<HashMap<String, Handlebars<'static>>>>,
    pub sign_map: Arc<Mutex<HashMap<String, bool>>>,
    pub chan_map: Arc<Mutex<HashMap<String, oneshot::Sender<()>>>>,
    pub child_map: Arc<Mutex<HashMap<String, tokio::process::Child>>>,
}

impl AppState {
    pub async fn from_env() -> Result<Self, ConfigError> {
        let env = EnvironmentVariables::from_env()?;
        Ok(Self {
            db: Database::connect(&*env.database_url)
                .await
                .map_err(|err| {
                    tracing::error!("failed to connect to the database: {:?}", err);
                    ConfigError::FailedDatabaseConnection
                })?
                .into(),
            redis_client: redis::Client::open(env.redis_url.as_ref())
                .map_err(|err| {
                    tracing::error!("failed to connect to Redis: {:?}", err);
                    ConfigError::FailedRedisConnection
                })?
                .into(),
            env: env.clone(),
            jwt_auth_keys: Keys::new(env.clone().jwt_secret.as_bytes()),
            jwt_refresh_keys: Keys::new(env.clone().jwt_refresh_secret.as_bytes()),
            template_cache: Arc::new(Mutex::new(HashMap::new())),
            sign_map: Arc::new(Mutex::new(HashMap::new())),
            chan_map: Arc::new(Mutex::new(HashMap::new())),
            child_map: Arc::new(Mutex::new(HashMap::new())),
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
    pub workerd_dir: Cow<'static, str>,
    pub worker_info_dir: Cow<'static, str>,
    pub workerd_bin_path: Cow<'static, str>,
}

impl EnvironmentVariables {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();

        fn get_env_var(key: &str) -> Result<String, ConfigError> {
            match dotenv::var(key) {
                Ok(value) => Ok(value),
                Err(err) => {
                    tracing::error!("missing {key}: {err}");
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
            workerd_dir: get_env_var("WORKERD_DIR")?.into(),
            worker_info_dir: get_env_var("WORKER_INFO_DIR")?.into(),
            workerd_bin_path: get_env_var("WORKERD_BIN_PATH")?.into(),
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
