use crate::model::backup_task::BackupTask;
use sqlx::SqlitePool;
use uuid::Uuid;

pub trait DatabaseOpsTrait {
    async fn check_table_exists(
        pool: SqlitePool,
        table_name: &str,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?)"
        )
            .bind(table_name)
            .fetch_one(&pool)
            .await?;
        Ok(result)
    }

    async fn create_backup_task_table(pool: SqlitePool) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE BackupTasks (
                id INTEGER PRIMARY KEY,
                uuid TEXT NOT NULL UNIQUE,
                source_path TEXT NOT NULL,
                destination_path TEXT NOT NULL,
                backup_type TEXT NOT NULL,
                comparison_mode TEXT NOT NULL,
                last_run_time DATETIME,
                next_run_time DATETIME,
                options TEXT,
            )
            "#,
        )
        .execute(&pool)
        .await?;
        Ok(())
    }

    async fn add_backup_task(pool: SqlitePool, backup_task: BackupTask) -> anyhow::Result<()> {
        Ok(())
    }

    async fn modify_backup_task(pool: SqlitePool, backup_task: BackupTask) -> anyhow::Result<()> {
        Ok(())
    }

    async fn remove_backup_task(pool: SqlitePool, uuid: Uuid) -> anyhow::Result<()> {
        Ok(())
    }

    // async fn create_file_info_table(pool: SqlitePool);
    // async fn delete_file_info_table(pool: SqlitePool);
    // async fn add_file_info(pool: SqlitePool);
    // async fn modify_file_info(pool: SqlitePool);
    // async fn delete_file_info(pool: SqlitePool);
}
