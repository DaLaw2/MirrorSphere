use crate::model::config::{Config, ConfigTable};
use crate::model::error::system::SystemError;
use crate::model::log::system::SystemLog;
use std::fs;
use std::sync::{OnceLock, RwLock as SyncRwLock};
use tokio::sync::RwLock as AsyncRwLock;

static SYNC_CONFIG: OnceLock<SyncRwLock<Config>> = OnceLock::new();
static ASYNC_CONFIG: OnceLock<AsyncRwLock<Config>> = OnceLock::new();

pub struct AppConfig;

impl AppConfig {
    pub async fn initialization() {
        SystemLog::Initializing.log();
        let config = Self::load_config();
        SYNC_CONFIG.get_or_init(|| SyncRwLock::new(config.clone()));
        ASYNC_CONFIG.get_or_init(move || AsyncRwLock::new(config));
        SystemLog::InitializeComplete.log();
    }

    fn load_config() -> Config {
        let config = match fs::read_to_string("./config.toml") {
            Ok(toml_string) => match toml::from_str::<ConfigTable>(&toml_string) {
                Ok(config_table) => config_table.config,
                Err(_) => panic!("{}", SystemError::InvalidConfig),
            },
            Err(_) => panic!("{}", SystemError::ConfigNotFound),
        };
        config
    }

    pub async fn fetch() -> Config {
        // Initialization has been ensured
        let lock = ASYNC_CONFIG.get().unwrap();
        lock.read().await.clone()
    }

    pub fn fetch_blocking() -> Config {
        // Initialization has been ensured
        let lock = SYNC_CONFIG.get().unwrap();
        // In extreme cases, a serious error occurs in the system
        lock.read().unwrap().clone()
    }

    pub async fn update(config: Config) {
        // Initialization has been ensured
        let lock = SYNC_CONFIG.get().unwrap();
        // There is no lock acquired multiple times, so this is safe
        *lock.write().unwrap() = config.clone();
        // Initialization has been ensured
        let lock = ASYNC_CONFIG.get().unwrap();
        *lock.write().await = config;
    }
}
