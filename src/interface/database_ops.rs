use crate::model::error::database::DatabaseError;
use crate::model::error::misc::MiscError;
use crate::model::error::Error;
use crate::model::task::BackupTask;
use crate::platform::constants::{DATABASE_LOCK_PATH, DATABASE_PATH};
use async_trait::async_trait;
use sqlx::SqlitePool;
use tokio::fs;
use tokio::fs::File;
use uuid::Uuid;

#[async_trait]
pub trait DatabaseOpsTrait {
    fn new(pool: SqlitePool) -> Self;

    fn get_pool(&self) -> SqlitePool;

    async fn exist_database() -> bool {
        fs::metadata(DATABASE_PATH).await.is_ok()
    }

    async fn create_database() -> Result<(), Error> {
        let _ = File::create(DATABASE_PATH)
            .await
            .map_err(|_| DatabaseError::CreateDatabaseFailed)?;
        Ok(())
    }

    async fn close_connection(&self) {
        let pool = self.get_pool();
        pool.close().await
    }

    async fn exist_table(&self, table_name: &str) -> bool {
        let pool = self.get_pool();
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?)",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .unwrap_or(false)
    }

    async fn create_backup_task_table(&self) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query(
            r#"
            CREATE TABLE BackupTasks (
                uuid BLOB PRIMARY KEY,
                source_path TEXT NOT NULL,
                destination_path TEXT NOT NULL,
                backup_type TEXT NOT NULL,
                comparison_mode TEXT NOT NULL,
                options TEXT NOT NULL,
                schedule INTEGER NOT NULL,
                last_run_time INTEGER,
                next_run_time INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|_| DatabaseError::StatementExecutionFailed)?;
        Ok(())
    }

    async fn add_backup_task(&self, backup_task: BackupTask) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query(
            r#"
        INSERT INTO BackupTasks (
            uuid,
            source_path,
            destination_path,
            backup_type,
            comparison_mode,
            options,
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        )
        .bind(backup_task.uuid)
        .bind(backup_task.source_path.to_string_lossy().to_string())
        .bind(backup_task.destination_path.to_string_lossy().to_string())
        .bind(
            serde_json::to_string(&backup_task.backup_type)
                .map_err(|_| MiscError::SerializeError)?,
        )
        .bind(
            serde_json::to_string(&backup_task.comparison_mode)
                .map_err(|_| MiscError::SerializeError)?,
        )
        .bind(serde_json::to_string(&backup_task.options).map_err(|_| MiscError::SerializeError)?)
        .execute(&pool)
        .await
        .map_err(|_| DatabaseError::StatementExecutionFailed)?;
        Ok(())
    }

    async fn modify_backup_task(&self, backup_task: BackupTask) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query(
            r#"
        UPDATE BackupTasks
        SET
            source_path = ?,
            destination_path = ?,
            backup_type = ?,
            comparison_mode = ?,
            options = ?,
        WHERE uuid = ?
        "#,
        )
        .bind(backup_task.source_path.to_string_lossy().to_string())
        .bind(backup_task.destination_path.to_string_lossy().to_string())
        .bind(
            serde_json::to_string(&backup_task.backup_type)
                .map_err(|_| MiscError::SerializeError)?,
        )
        .bind(
            serde_json::to_string(&backup_task.comparison_mode)
                .map_err(|_| MiscError::SerializeError)?,
        )
        .bind(serde_json::to_string(&backup_task.options).map_err(|_| MiscError::SerializeError)?)
        .bind(backup_task.uuid)
        .execute(&pool)
        .await
        .map_err(|_| DatabaseError::StatementExecutionFailed)?;
        Ok(())
    }

    async fn remove_backup_task(&self, uuid: Uuid) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query("DELETE FROM BackupTasks WHERE uuid = ?")
            .bind(uuid)
            .execute(&pool)
            .await
            .map_err(|_| DatabaseError::StatementExecutionFailed)?;
        Ok(())
    }
}
