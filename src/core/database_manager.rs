use std::ops::Deref;
use crate::interface::database_ops::DatabaseOpsTrait;
use crate::platform::constants::*;
use crate::platform::database_ops::DatabaseOps;
use crate::utils::log_entry::database::DatabaseEntry;
use crate::utils::log_entry::system::SystemEntry;
use sqlx::SqlitePool;
use std::sync::OnceLock;
use tracing::{error, info, trace};

static DATABASE_MANAGER: OnceLock<DatabaseManager> = OnceLock::new();

#[derive(Debug)]
pub struct DatabaseManager {
    ops: DatabaseOps,
}

impl DatabaseManager {
    pub async fn initialization() {
        info!("{}", SystemEntry::Initializing);
        if let Err(err) = DatabaseOps::lock_database().await {
            panic!("{}", err);
        }
        if !DatabaseOps::exist_database().await {
            if let Err(err) = DatabaseOps::create_database().await {
                panic!("{}", err);
            }
        }
        let instance = match SqlitePool::connect(DATABASE_URL).await {
            Ok(pool) => {
                info!("{}", DatabaseEntry::DatabaseConnectSuccess);
                DatabaseManager { ops: DatabaseOps::new(pool) }
            }
            Err(err) => {
                trace!(?err);
                panic!("{}", DatabaseEntry::DatabaseConnectFailed);
            }
        };
        if !instance.exist_table("BackupTasks").await {
            if let Err(err) = instance.create_backup_task_table().await {
                error!("{}", err);
            }
        }
        DATABASE_MANAGER.set(instance).unwrap();
        info!("{}", SystemEntry::InitializeComplete);
    }

    pub async fn terminate() {
        let _ = DatabaseOps::unlock_database().await;
    }

    pub fn instance() -> &'static DatabaseManager {
        DATABASE_MANAGER.get().unwrap()
    }
}

impl Deref for DatabaseManager {
    type Target = DatabaseOps;

    fn deref(&self) -> &Self::Target {
        &self.ops
    }
}
