use crate::core::config_manager::ConfigManager;
use crate::utils::log_entry::database::DatabaseEntry;
use crate::utils::log_entry::system::SystemEntry;
use lazy_static::lazy_static;
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::sync::OnceCell;
use tokio::time::sleep;
use tracing::{error, info, trace};

static DATABASE_MANAGER: OnceCell<DatabaseManager> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn initialization() {
        info!("{}", SystemEntry::Initializing);
        let config = ConfigManager::now().await;
        #[cfg(target_os = "windows")]
        let database_url = "sqlite://.\\mirrorSphere.db";
        #[cfg(target_os = "linux")]
        let database_url = "sqlite://./mirrorSphere.db";
        loop {
            match SqlitePool::connect(database_url).await {
                Ok(pool) => {
                    let instance = DatabaseManager { pool };
                    // There are no re-initialization or multiple-initialization issues, so it's safe
                    DATABASE_MANAGER.set(instance).unwrap();
                    info!("{}", DatabaseEntry::DatabaseConnectSuccess);
                    break;
                }
                Err(err) => {
                    error!("{}", DatabaseEntry::DatabaseConnectFailed);
                    trace!(?err);
                    sleep(Duration::from_secs(config.retry_interval)).await;
                }
            }
        }
        info!("{}", SystemEntry::InitializeComplete);
    }

    pub async fn instance(&self) -> SqlitePool {
        // Initialization has been ensured
        DATABASE_MANAGER.get().unwrap().pool.clone()
    }

    pub async fn add_task(task: Task) -> Result<(), >
}
