use crate::interface::database_ops::DatabaseOps as DatabaseOpsTrait;
use crate::model::backup_task::BackupTask;
use crate::platform::constants::*;
use crate::platform::database_ops::DatabaseOps;
use crate::utils::log_entry::database::DatabaseEntry;
use crate::utils::log_entry::system::SystemEntry;
use sqlx::SqlitePool;
use std::sync::OnceLock;
use tracing::{error, info, trace};
use uuid::Uuid;

static DATABASE_MANAGER: OnceLock<DatabaseManager> = OnceLock::new();

#[derive(Debug)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn initialization() {
        info!("{}", SystemEntry::Initializing);
        if let Err(err) = Self::lock_database().await {
            panic!("{}", err);
        }
        if !Self::exist_database().await {
            if let Err(err) = Self::create_database().await {
                panic!("{}", err);
            }
        }
        match SqlitePool::connect(DATABASE_URL).await {
            Ok(pool) => {
                let instance = DatabaseManager { pool };
                // There are no re-initialization or multiple-initialization issues, so it's safe
                DATABASE_MANAGER.set(instance).unwrap();
                info!("{}", DatabaseEntry::DatabaseConnectSuccess);
            }
            Err(err) => {
                trace!(?err);
                panic!("{}", DatabaseEntry::DatabaseConnectFailed);
            }
        }
        if !Self::exist_table("BackupTasks").await {
            if let Err(err) = Self::create_backup_task_table().await {
                error!("{}", err);
            }
        }
        info!("{}", SystemEntry::InitializeComplete);
    }

    pub async fn terminate() {
        let _ = Self::unlock_database().await;
    }

    pub fn instance() -> SqlitePool {
        DATABASE_MANAGER.get().unwrap().pool.clone()
    }

    pub async fn exist_database() -> bool {
        DatabaseOps::exist_database().await
    }

    pub async fn create_database() -> anyhow::Result<()> {
        DatabaseOps::create_database().await
    }

    pub async fn lock_database() -> anyhow::Result<()> {
        DatabaseOps::lock_database().await
    }

    pub async fn unlock_database() -> anyhow::Result<()> {
        DatabaseOps::unlock_database().await
    }

    pub async fn exist_table(table_name: &str) -> bool {
        let pool = DatabaseManager::instance();
        DatabaseOps::exist_table(pool, table_name).await
    }

    pub async fn create_backup_task_table() -> anyhow::Result<()> {
        let pool = DatabaseManager::instance();
        DatabaseOps::create_backup_task_table(pool).await
    }

    pub async fn add_backup_task(backup_task: BackupTask) -> anyhow::Result<()> {
        let pool = DatabaseManager::instance();
        DatabaseOps::add_backup_task(pool, backup_task).await
    }

    pub async fn modify_backup_task(backup_task: BackupTask) -> anyhow::Result<()> {
        let pool = DatabaseManager::instance();
        DatabaseOps::modify_backup_task(pool, backup_task).await
    }

    pub async fn remove_backup_task(uuid: Uuid) -> anyhow::Result<()> {
        let pool = DatabaseManager::instance();
        DatabaseOps::remove_backup_task(pool, uuid).await
    }
}
