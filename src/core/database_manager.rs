use crate::model::backup_schedule::{BackupSchedule, ScheduleState};
use crate::model::error::database::DatabaseError;
use crate::model::error::misc::MiscError;
use crate::model::error::Error;
use crate::model::log::database::DatabaseLog;
use crate::model::log::system::SystemLog;
use crate::platform::constants::*;
use macros::log;
use sqlx::{Row, SqlitePool};
use tokio::fs;
use tokio::fs::File;
use uuid::Uuid;

#[derive(Debug)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn new() -> Result<Self, Error> {
        log!(SystemLog::Initializing);
        if !Self::exist_database().await {
            Self::create_database().await?;
        }
        let pool = SqlitePool::connect(DATABASE_URL)
            .await
            .map_err(|err| DatabaseError::DatabaseConnectFailed(err))?;
        log!(DatabaseLog::DatabaseConnectSuccess);
        let database_manager = Self {
            pool,
        };
        if !database_manager.exist_table("BackupSchedules").await {
            database_manager.create_backup_schedule_table().await?;
        }
        log!(SystemLog::InitializeComplete);
        Ok(database_manager)
    }

    fn get_pool(&self) -> SqlitePool {
        self.pool.clone()
    }

    pub async fn close_connection(&self) {
        let pool = self.get_pool();
        pool.close().await
    }

    async fn exist_database() -> bool {
        fs::metadata(DATABASE_PATH).await.is_ok()
    }

    async fn create_database() -> Result<(), Error> {
        let _ = File::create(DATABASE_PATH)
            .await
            .map_err(|err| DatabaseError::CreateDatabaseFailed(err))?;
        Ok(())
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

    async fn create_backup_schedule_table(&self) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query(
            r#"
            CREATE TABLE BackupSchedules (
                uuid BLOB PRIMARY KEY,
                name TEXT NOT NULL,
                state INTEGER NOT NULL,
                source_path TEXT NOT NULL,
                destination_path TEXT NOT NULL,
                schedule_type TEXT NOT NULL,
                last_run_time INTEGER,
                next_run_time INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
            .execute(&pool)
            .await
            .map_err(|err| DatabaseError::StatementExecutionFailed(err))?;
        Ok(())
    }

    pub async fn add_backup_schedule(&self, backup_schedule: &BackupSchedule) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query(
            r#"
            INSERT INTO BackupSchedules (
                uuid,
                name,
                state,
                source_path,
                destination_path,
                schedule_type,
                last_run_time,
                next_run_time,
                created_at,
                updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
            .bind(backup_schedule.uuid.as_bytes().as_slice())
            .bind(&backup_schedule.name)
            .bind(backup_schedule.state as i32)
            .bind(backup_schedule.source_path.to_string_lossy().to_string())
            .bind(backup_schedule.destination_path.to_string_lossy().to_string())
            .bind(
                serde_json::to_string(&backup_schedule.schedule_type)
                    .map_err(|err| MiscError::SerializeError(err))?,
            )
            .bind(backup_schedule.last_run_time)
            .bind(backup_schedule.next_run_time)
            .bind(backup_schedule.created_at)
            .bind(backup_schedule.updated_at)
            .execute(&pool)
            .await
            .map_err(|err| DatabaseError::StatementExecutionFailed(err))?;
        Ok(())
    }

    pub async fn modify_backup_schedule(&self, backup_schedule: &BackupSchedule) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query(
            r#"
            UPDATE BackupSchedules
            SET
                name = ?,
                state = ?,
                source_path = ?,
                destination_path = ?,
                schedule_type = ?,
                last_run_time = ?,
                next_run_time = ?,
                updated_at = ?
            WHERE uuid = ?
            "#,
        )
            .bind(&backup_schedule.name)
            .bind(backup_schedule.state as i32)
            .bind(backup_schedule.source_path.to_string_lossy().to_string())
            .bind(backup_schedule.destination_path.to_string_lossy().to_string())
            .bind(
                serde_json::to_string(&backup_schedule.schedule_type)
                    .map_err(|err| MiscError::SerializeError(err))?,
            )
            .bind(backup_schedule.last_run_time)
            .bind(backup_schedule.next_run_time)
            .bind(backup_schedule.updated_at)
            .bind(backup_schedule.uuid.as_bytes().as_slice())
            .execute(&pool)
            .await
            .map_err(|err| DatabaseError::StatementExecutionFailed(err))?;
        Ok(())
    }

    pub async fn remove_backup_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query("DELETE FROM BackupSchedules WHERE uuid = ?")
            .bind(uuid.as_bytes().as_slice())
            .execute(&pool)
            .await
            .map_err(|err| DatabaseError::StatementExecutionFailed(err))?;
        Ok(())
    }

    pub async fn get_backup_schedule(&self, uuid: Uuid) -> Result<Option<BackupSchedule>, Error> {
        let pool = self.get_pool();
        let row = sqlx::query(
            r#"
            SELECT uuid, name, state, source_path, destination_path, schedule_type,
                   last_run_time, next_run_time, created_at, updated_at
            FROM BackupSchedules
            WHERE uuid = ?
            "#,
        )
            .bind(uuid.as_bytes().as_slice())
            .fetch_optional(&pool)
            .await
            .map_err(|err| DatabaseError::StatementExecutionFailed(err))?;

        if let Some(row) = row {
            let uuid_bytes: Vec<u8> = row.get("uuid");
            let uuid = Uuid::from_slice(&uuid_bytes).map_err(|_| DatabaseError::DataCorrupted)?;

            let state_value: i32 = row.get("state");
            let state = match state_value {
                0 => ScheduleState::Active,
                1 => ScheduleState::Paused,
                2 => ScheduleState::Disabled,
                _ => Err(DatabaseError::DataCorrupted)?,
            };

            let schedule_type_str: String = row.get("schedule_type");
            let schedule_type = serde_json::from_str(&schedule_type_str)
                .map_err(|err| MiscError::DeserializeError(err))?;

            Ok(Some(BackupSchedule {
                uuid,
                name: row.get("name"),
                state,
                source_path: row.get::<String, _>("source_path").into(),
                destination_path: row.get::<String, _>("destination_path").into(),
                schedule_type,
                last_run_time: row.get("last_run_time"),
                next_run_time: row.get("next_run_time"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_backup_schedules(&self) -> Result<Vec<BackupSchedule>, Error> {
        let pool = self.get_pool();
        let rows = sqlx::query(
            r#"
            SELECT uuid, name, state, source_path, destination_path, schedule_type,
                   last_run_time, next_run_time, created_at, updated_at
            FROM BackupSchedules
            "#,
        )
            .fetch_all(&pool)
            .await
            .map_err(|err| DatabaseError::StatementExecutionFailed(err))?;

        let mut schedules = Vec::new();
        for row in rows {
            let uuid_bytes: Vec<u8> = row.get("uuid");
            let uuid = Uuid::from_slice(&uuid_bytes).map_err(|_| DatabaseError::DataCorrupted)?;

            let state_value: i32 = row.get("state");
            let state = match state_value {
                0 => ScheduleState::Active,
                1 => ScheduleState::Paused,
                2 => ScheduleState::Disabled,
                _ => return Err(DatabaseError::DataCorrupted.into()),
            };

            let schedule_type_str: String = row.get("schedule_type");
            let schedule_type = serde_json::from_str(&schedule_type_str)
                .map_err(|err| MiscError::DeserializeError(err))?;

            schedules.push(BackupSchedule {
                uuid,
                name: row.get("name"),
                state,
                source_path: row.get::<String, _>("source_path").into(),
                destination_path: row.get::<String, _>("destination_path").into(),
                schedule_type,
                last_run_time: row.get("last_run_time"),
                next_run_time: row.get("next_run_time"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(schedules)
    }

    pub async fn get_ready_schedules(&self) -> Result<Vec<BackupSchedule>, Error> {
        let now = chrono::Utc::now().timestamp();
        let pool = self.get_pool();
        let rows = sqlx::query(
            r#"
            SELECT uuid, name, state, source_path, destination_path, schedule_type,
                   last_run_time, next_run_time, created_at, updated_at
            FROM BackupSchedules
            WHERE state = 0 AND next_run_time IS NOT NULL AND next_run_time <= ?
            "#,
        )
            .bind(now)
            .fetch_all(&pool)
            .await
            .map_err(|err| DatabaseError::StatementExecutionFailed(err))?;

        let mut schedules = Vec::new();
        for row in rows {
            let uuid_bytes: Vec<u8> = row.get("uuid");
            let uuid = Uuid::from_slice(&uuid_bytes).map_err(|_| DatabaseError::DataCorrupted)?;

            let schedule_type_str: String = row.get("schedule_type");
            let schedule_type = serde_json::from_str(&schedule_type_str)
                .map_err(|err| MiscError::DeserializeError(err))?;

            schedules.push(BackupSchedule {
                uuid,
                name: row.get("name"),
                state: ScheduleState::Active,
                source_path: row.get::<String, _>("source_path").into(),
                destination_path: row.get::<String, _>("destination_path").into(),
                schedule_type,
                last_run_time: row.get("last_run_time"),
                next_run_time: row.get("next_run_time"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(schedules)
    }
}
