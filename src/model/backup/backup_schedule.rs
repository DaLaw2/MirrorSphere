use crate::model::backup::backup_execution::*;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleState {
    Active,
    Paused,
    Disabled,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleInterval {
    Once,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone)]
pub struct BackupSchedule {
    pub uuid: Uuid,
    pub name: String,
    pub state: ScheduleState,
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub backup_type: BackupType,
    pub comparison_mode: Option<ComparisonMode>,
    pub options: BackupOptions,
    pub interval: ScheduleInterval,
    pub last_run_time: Option<NaiveDateTime>,
    pub next_run_time: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl BackupSchedule {
    pub fn to_execution(&self) -> BackupExecution {
        BackupExecution {
            uuid: self.uuid,
            state: BackupState::Pending,
            source_path: self.source_path.clone(),
            destination_path: self.destination_path.clone(),
            backup_type: if self.last_run_time.is_some() {
                self.backup_type
            } else {
                BackupType::Full
            },
            comparison_mode: self.comparison_mode,
            options: self.options,
        }
    }
}
