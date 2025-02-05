use crate::core::config_manager::ConfigManager;
use crate::model::backup_task::BackupTask;
use crate::platform::constants::*;
use crate::platform::database_ops::DatabaseOps;
use crate::utils::file_lock::FileLock;
use crate::utils::log_entry::database::DatabaseEntry;
use crate::utils::log_entry::system::SystemEntry;
use sqlx::SqlitePool;
use std::future::Future;
use std::io;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::fs;
use tokio::fs::File;
use tokio::sync::OnceCell;
use tokio::time::sleep;
use tracing::{error, info, trace, Instrument};
use uuid::Uuid;
use crate::interface::database_ops::DatabaseOpsTrait;

static DATABASE_MANAGER: OnceLock<DatabaseManager> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn initialization() {
        info!("{}", SystemEntry::Initializing);
        if let Err(err) = Self::lock_database().await {
            error!("{}", err);
            panic!("{}", err);
        }
        if !Self::exist_database().await {
            if let Err(err) = Self::create_database().await {
                error!("{}", err);
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
                error!("{}", DatabaseEntry::DatabaseConnectFailed);
                trace!(?err);
                panic!("{}", DatabaseEntry::DatabaseConnectFailed);
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
        fs::metadata(DATABASE_PATH).await.is_ok()
    }

    pub async fn create_database() -> anyhow::Result<()> {
        let _ = File::create(DATABASE_PATH)
            .await
            .map_err(|e| DatabaseEntry::CreateDatabaseFailed)?;
        Ok(())
    }

    pub async fn lock_database() -> anyhow::Result<()> {
        if !fs::metadata(DATABASE_LOCK_PATH).await.is_ok() {
            File::create(&DATABASE_LOCK_PATH)
                .await
                .map_err(|_| DatabaseEntry::LockDatabaseFailed)?;
            Ok(())
        } else {
            Err(DatabaseEntry::LockDatabaseFailed)?
        }
    }

    pub async fn unlock_database() -> anyhow::Result<()> {
        fs::remove_file(&DATABASE_LOCK_PATH)
            .await
            .map_err(|_| DatabaseEntry::UnlockDatabaseFailed)?;
        Ok(())
    }

    pub async fn check_table_exists(table_name: &str) -> anyhow::Result<bool> {
        let pool = DatabaseManager::instance();
        DatabaseOpsTrait::check_table_exists(pool, table_name).await
    }

    pub async fn create_backup_task_table() -> anyhow::Result<()> {
        let pool = DatabaseManager::instance();
        DatabaseOpsTrait::create_backup_task_table(pool).await
    }

    pub async fn add_backup_task(backup_task: BackupTask) -> anyhow::Result<()> {
        let pool = DatabaseManager::instance();
        DatabaseOpsTrait::add_backup_task(pool, backup_task).await
    }

    pub async fn modify_backup_task(backup_task: BackupTask) -> anyhow::Result<()> {
        let pool = DatabaseManager::instance();
        DatabaseOpsTrait::modify_backup_task(pool, backup_task).await
    }

    pub async fn remove_backup_task(uuid: Uuid) -> anyhow::Result<()> {
        let pool = DatabaseManager::instance();
        DatabaseOpsTrait::remove_backup_task(pool, uuid).await
    }
}
