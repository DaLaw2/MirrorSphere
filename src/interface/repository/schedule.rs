use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::model::core::schedule::backup_schedule::BackupSchedule;
use crate::model::error::Error;
use crate::model::error::database::DatabaseError;
use crate::model::error::misc::MiscError;
use sqlx::Row;
use uuid::Uuid;

pub trait ScheduleRepository {
    async fn create_backup_schedule_table(&self) -> Result<(), Error>;
    async fn create_backup_schedule(&self, backup_schedule: &BackupSchedule) -> Result<(), Error>;
    async fn modify_backup_schedule(&self, backup_schedule: &BackupSchedule) -> Result<(), Error>;
    async fn remove_backup_schedule(&self, uuid: Uuid) -> Result<(), Error>;
    async fn get_backup_schedule(&self, uuid: Uuid) -> Result<Option<BackupSchedule>, Error>;
    async fn get_all_backup_schedules(&self) -> Result<Vec<BackupSchedule>, Error>;
}

impl ScheduleRepository for DatabaseManager {
    async fn create_backup_schedule_table(&self) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query(
            r#"
            CREATE TABLE BackupSchedules (
                uuid BLOB PRIMARY KEY,
                name TEXT NOT NULL,
                state TEXT NOT NULL,
                source_path TEXT NOT NULL,
                destination_path TEXT NOT NULL,
                backup_type TEXT NOT NULL,
                comparison_mode TEXT,
                options TEXT NOT NULL,
                interval TEXT NOT NULL,
                last_run_time TEXT,
                next_run_time TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
            .execute(&pool)
            .await
            .map_err(DatabaseError::StatementExecutionFailed)?;
        Ok(())
    }

    async fn create_backup_schedule(&self, backup_schedule: &BackupSchedule) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query(
            r#"
            INSERT INTO BackupSchedules (
                uuid,
                name,
                state,
                source_path,
                destination_path,
                backup_type,
                comparison_mode,
                options,
                interval,
                last_run_time,
                next_run_time,
                created_at,
                updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
            .bind(backup_schedule.uuid.as_bytes().as_slice())
            .bind(&backup_schedule.name)
            .bind(
                serde_json::to_string(&backup_schedule.state)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(backup_schedule.source_path.to_string_lossy().to_string())
            .bind(backup_schedule.destination_path.to_string_lossy().to_string())
            .bind(
                serde_json::to_string(&backup_schedule.backup_type)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(
                serde_json::to_string(&backup_schedule.comparison_mode)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(
                serde_json::to_string(&backup_schedule.options)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(
                serde_json::to_string(&backup_schedule.interval)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(backup_schedule.last_run_time)
            .bind(backup_schedule.next_run_time)
            .bind(backup_schedule.created_at)
            .bind(backup_schedule.updated_at)
            .execute(&pool)
            .await
            .map_err(DatabaseError::StatementExecutionFailed)?;
        Ok(())
    }

    async fn modify_backup_schedule(&self, backup_schedule: &BackupSchedule) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query(
            r#"
            UPDATE BackupSchedules
            SET
                name = ?,
                state = ?,
                source_path = ?,
                destination_path = ?,
                backup_type = ?,
                comparison_mode = ?,
                options = ?,
                interval = ?,
                last_run_time = ?,
                next_run_time = ?,
                created_at = ?,
                updated_at  = ?
            WHERE uuid = ?
            "#,
        )
            .bind(backup_schedule.uuid.as_bytes().as_slice())
            .bind(&backup_schedule.name)
            .bind(
                serde_json::to_string(&backup_schedule.state)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(backup_schedule.source_path.to_string_lossy().to_string())
            .bind(backup_schedule.destination_path.to_string_lossy().to_string())
            .bind(
                serde_json::to_string(&backup_schedule.backup_type)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(
                serde_json::to_string(&backup_schedule.comparison_mode)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(
                serde_json::to_string(&backup_schedule.options)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(
                serde_json::to_string(&backup_schedule.interval)
                    .map_err(MiscError::SerializeError)?,
            )
            .bind(backup_schedule.last_run_time)
            .bind(backup_schedule.next_run_time)
            .bind(backup_schedule.created_at)
            .bind(backup_schedule.updated_at)
            .execute(&pool)
            .await
            .map_err(DatabaseError::StatementExecutionFailed)?;
        Ok(())
    }

    async fn remove_backup_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        let pool = self.get_pool();
        sqlx::query("DELETE FROM BackupSchedules WHERE uuid = ?")
            .bind(uuid.as_bytes().as_slice())
            .execute(&pool)
            .await
            .map_err(DatabaseError::StatementExecutionFailed)?;
        Ok(())
    }

    async fn get_backup_schedule(&self, uuid: Uuid) -> Result<Option<BackupSchedule>, Error> {
        let pool = self.get_pool();
        let row = sqlx::query(
            r#"
            SELECT
                uuid,
                name,
                state,
                source_path,
                destination_path,
                backup_type,
                comparison_mode,
                options,
                "interval",
                last_run_time,
                next_run_time,
                created_at,
                updated_at
            FROM BackupSchedules
            WHERE uuid = ?
            "#,
        )
            .bind(uuid.as_bytes().as_slice())
            .fetch_optional(&pool)
            .await
            .map_err(DatabaseError::StatementExecutionFailed)?;

        if let Some(row) = row {
            let uuid_bytes: Vec<u8> = row.get("uuid");
            let uuid = Uuid::from_slice(&uuid_bytes).map_err(|_| DatabaseError::DataCorrupted)?;

            let state_str: String = row.get("state");
            let state = serde_json::from_str(&state_str)
                .map_err(MiscError::DeserializeError)?;

            let backup_type_str: String = row.get("backup_type");
            let backup_type = serde_json::from_str(&backup_type_str)
                .map_err(MiscError::DeserializeError)?;

            let comparison_mode_str: String = row.get("comparison_mode");
            let comparison_mode = serde_json::from_str(&comparison_mode_str)
                .map_err(MiscError::DeserializeError)?;

            let options_str: String = row.get("options");
            let options = serde_json::from_str(&options_str)
                .map_err(MiscError::DeserializeError)?;

            let interval_str: String = row.get("interval");
            let interval = serde_json::from_str(&interval_str)
                .map_err(MiscError::DeserializeError)?;

            Ok(Some(BackupSchedule {
                uuid,
                name: row.get("name"),
                state,
                source_path: row.get::<String, _>("source_path").into(),
                destination_path: row.get::<String, _>("destination_path").into(),
                backup_type,
                comparison_mode,
                options,
                interval,
                last_run_time: row.get("last_run_time"),
                next_run_time: row.get("next_run_time"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_all_backup_schedules(&self) -> Result<Vec<BackupSchedule>, Error> {
        let pool = self.get_pool();
        let rows = sqlx::query(
            r#"
            SELECT
                uuid,
                name,
                state,
                source_path,
                destination_path,
                backup_type,
                comparison_mode,
                options,
                "interval",
                last_run_time,
                next_run_time,
                created_at,
                updated_at
            FROM BackupSchedules
            "#,
        )
            .fetch_all(&pool)
            .await
            .map_err(DatabaseError::StatementExecutionFailed)?;

        let mut schedules = Vec::new();
        for row in rows {
            let uuid_bytes: Vec<u8> = row.get("uuid");
            let uuid = Uuid::from_slice(&uuid_bytes).map_err(|_| DatabaseError::DataCorrupted)?;

            let state_str: String = row.get("state");
            let state = serde_json::from_str(&state_str)
                .map_err(MiscError::DeserializeError)?;

            let backup_type_str: String = row.get("backup_type");
            let backup_type = serde_json::from_str(&backup_type_str)
                .map_err(MiscError::DeserializeError)?;

            let comparison_mode_str: String = row.get("comparison_mode");
            let comparison_mode = serde_json::from_str(&comparison_mode_str)
                .map_err(MiscError::DeserializeError)?;

            let options_str: String = row.get("options");
            let options = serde_json::from_str(&options_str)
                .map_err(MiscError::DeserializeError)?;

            let interval_str: String = row.get("interval");
            let interval = serde_json::from_str(&interval_str)
                .map_err(MiscError::DeserializeError)?;

            schedules.push(BackupSchedule {
                uuid,
                name: row.get("name"),
                state,
                source_path: row.get::<String, _>("source_path").into(),
                destination_path: row.get::<String, _>("destination_path").into(),
                backup_type,
                comparison_mode,
                options,
                interval,
                last_run_time: row.get("last_run_time"),
                next_run_time: row.get("next_run_time"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(schedules)
    }
}
