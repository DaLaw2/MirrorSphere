use crate::interface::database_ops::DatabaseOpsTrait;
use crate::platform::constants::*;
use crate::platform::database_ops::DatabaseOps;
use crate::model::log::database::DatabaseLog;
use crate::model::log::system::SystemLog;
use sqlx::SqlitePool;
use std::ops::Deref;
use std::sync::OnceLock;
use tracing::{info, trace};
use crate::model::error::database::DatabaseError;

static DATABASE_MANAGER: OnceLock<DatabaseManager> = OnceLock::new();

#[derive(Debug)]
pub struct DatabaseManager {
    ops: DatabaseOps,
}

impl DatabaseManager {
    pub async fn initialization() {
        info!("{}", SystemLog::Initializing);
        DatabaseOps::lock_database().await.unwrap();
        if !DatabaseOps::exist_database().await {
            DatabaseOps::create_database().await.unwrap();
        }
        let instance = match SqlitePool::connect(DATABASE_URL).await {
            Ok(pool) => {
                info!("{}", DatabaseLog::DatabaseConnectSuccess);
                DatabaseManager {
                    ops: DatabaseOps::new(pool),
                }
            }
            Err(err) => {
                trace!(?err);
                panic!("{}", DatabaseError::DatabaseConnectFailed);
            }
        };
        if !instance.exist_table("BackupTasks").await {
            instance.create_backup_task_table().await.unwrap();
        }
        DATABASE_MANAGER.set(instance).unwrap();
        info!("{}", SystemLog::InitializeComplete);
    }

    pub fn instance() -> &'static DatabaseManager {
        // Initialization has been ensured
        DATABASE_MANAGER.get().unwrap()
    }

    pub async fn terminate() {
        let instance = DatabaseManager::instance();
        instance.close_connection().await;
        let _ = DatabaseOps::unlock_database().await;
    }
}

impl Deref for DatabaseManager {
    type Target = DatabaseOps;

    fn deref(&self) -> &Self::Target {
        &self.ops
    }
}
