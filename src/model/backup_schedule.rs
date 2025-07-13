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
pub enum ScheduleType {
    Once,
    Interval(u64),
    Daily,
    Weekly,
    Monthly,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupSchedule {
    pub uuid: Uuid,
    pub name: String,
    pub state: ScheduleState,
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub schedule_type: ScheduleType,
    pub last_run_time: Option<i64>,
    pub next_run_time: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}
