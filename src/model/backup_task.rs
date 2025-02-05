use crate::model::backup_options::BackupOptions;
use crate::model::backup_type::BackupType;
use crate::model::comparison_mode::ComparisonMode;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

pub struct BackupTask {
    pub uuid: Uuid,
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub backup_type: BackupType,
    pub comparison_mode: ComparisonMode,
    pub options: BackupOptions,
    pub schedule: bool,
    pub last_run_time: Option<SystemTime>,
    pub next_run_time: Option<SystemTime>,
}
