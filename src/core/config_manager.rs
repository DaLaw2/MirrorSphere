use crate::model::config::{Config, ConfigTable};
use crate::utils::log_entry::system::SystemEntry;
use lazy_static::lazy_static;
use std::cell::OnceCell;
use std::fs;
use std::sync::RwLock as SyncRwLock;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{error, info};

static SYNC_CONFIG: OnceCell<SyncRwLock<Config>> = OnceCell::new();
static ASYNC_CONFIG: OnceCell<AsyncRwLock<Config>> = OnceCell::new();

pub struct ConfigManager {}

impl ConfigManager {
    pub async fn initialization() {
        info!("{}", SystemEntry::Initializing);
        let config = Self::load_config();
        SYNC_CONFIG.get_or_init(|| SyncRwLock::new(config.clone()));
        ASYNC_CONFIG.get_or_init(move || AsyncRwLock::new(config));
        info!("{}", SystemEntry::InitializeComplete);
    }

    fn load_config() -> Config {
        let config = match fs::read_to_string("./config.toml") {
            Ok(toml_string) => match toml::from_str::<ConfigTable>(&toml_string) {
                Ok(config_table) => {
                    let config = config_table.config;
                    if !Self::validate(&config) {
                        error!("{}", SystemEntry::InvalidConfig);
                        panic!("{}", SystemEntry::InvalidConfig);
                    }
                    config
                }
                Err(_) => {
                    error!("{}", SystemEntry::InvalidConfig);
                    panic!("{}", SystemEntry::InvalidConfig);
                }
            },
            Err(_) => {
                error!("{}", SystemEntry::ConfigNotFound);
                panic!("{}", SystemEntry::ConfigNotFound);
            }
        };
        config
    }

    pub fn now_blocking() -> Config {
        // Initialization has been ensured
        let lock = SYNC_CONFIG.get().unwrap();
        // There is no lock acquired multiple times, so this is safe
        lock.read().unwrap().clone()
    }

    pub async fn now() -> Config {
        // Initialization has been ensured
        let lock = ASYNC_CONFIG.get().unwrap();
        // There is no lock acquired multiple times, so this is safe
        lock.read().await.clone()
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

    fn validate(config: &Config) -> bool {
        Self::validate_second(config.retry_interval)
    }

    fn validate_second(second: u64) -> bool {
        second <= 3600
    }
}
