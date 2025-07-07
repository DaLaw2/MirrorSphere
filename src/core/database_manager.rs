use crate::interface::database_ops::DatabaseOpsTrait;
use crate::model::error::database::DatabaseError;
use crate::model::error::Error;
use crate::model::log::database::DatabaseLog;
use crate::model::log::system::SystemLog;
use crate::platform::constants::*;
use crate::platform::database_ops::DatabaseOps;
use sqlx::SqlitePool;
use std::ops::Deref;
use crate::log;

#[derive(Debug)]
pub struct DatabaseManager {
    ops: DatabaseOps,
}

impl DatabaseManager {
    pub async fn new() -> Result<Self, Error> {
        log!(SystemLog::Initializing);
        if !DatabaseOps::exist_database().await {
            DatabaseOps::create_database().await?;
        }
        let pool = SqlitePool::connect(DATABASE_URL)
            .await
            .map_err(|err| DatabaseError::DatabaseConnectFailed(err))?;
        log!(DatabaseLog::DatabaseConnectSuccess);
        let database_manager = Self {
            ops: DatabaseOps::new(pool),
        };
        if !database_manager.exist_table("BackupTasks").await {
            database_manager.create_backup_task_table().await?;
        }
        log!(SystemLog::InitializeComplete);
        Ok(database_manager)
    }

    pub async fn terminate(&self) {
        self.close_connection().await;
    }
}

impl Deref for DatabaseManager {
    type Target = DatabaseOps;

    fn deref(&self) -> &Self::Target {
        &self.ops
    }
}
