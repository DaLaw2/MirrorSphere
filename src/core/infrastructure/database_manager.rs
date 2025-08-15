use crate::interface::repository::schedule::ScheduleRepository;
use crate::model::error::database::DatabaseError;
use crate::model::error::Error;
use crate::model::log::database::DatabaseLog;
use crate::model::log::system::SystemLog;
use crate::platform::constants::*;
use crate::utils::database_lock::DatabaseLock;
use macros::log;
use sqlx::SqlitePool;
use tokio::fs;
use tokio::fs::File;

#[derive(Debug)]
pub struct DatabaseManager {
    pool: SqlitePool,
    _lock: DatabaseLock,
}

impl DatabaseManager {
    pub async fn new() -> Result<Self, Error> {
        log!(SystemLog::Initializing);
        let lock = DatabaseLock::acquire().await?;
        if !Self::exist_database().await {
            Self::create_database().await?;
        }
        let pool = SqlitePool::connect(DATABASE_URL)
            .await
            .map_err(DatabaseError::DatabaseConnectFailed)?;
        log!(DatabaseLog::DatabaseConnectSuccess);
        let database_manager = Self { pool, _lock: lock };
        if !database_manager.exist_table("BackupSchedules").await {
            database_manager.create_backup_schedule_table().await?;
        }
        log!(SystemLog::InitializeComplete);
        Ok(database_manager)
    }

    pub fn get_pool(&self) -> SqlitePool {
        self.pool.clone()
    }

    pub async fn exist_database() -> bool {
        fs::metadata(DATABASE_PATH).await.is_ok()
    }

    pub async fn create_database() -> Result<(), Error> {
        let _ = File::create(DATABASE_PATH)
            .await
            .map_err(DatabaseError::CreateDatabaseFailed)?;
        Ok(())
    }

    pub async fn exist_table(&self, table_name: &str) -> bool {
        let pool = self.get_pool();
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?)",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .unwrap_or(false)
    }
}
