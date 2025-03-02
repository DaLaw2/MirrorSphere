use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BackupState {
    Running,
    Suspended,
    Stopped,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BackupType {
    Full,
    Incremental,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HashType {
    MD5,
    SHA3,
    SHA256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ComparisonMode {
    // Compare size and modify time
    Quick,
    // Quick + compare regular file attr
    Standard,
    // Standard + compare file checksum
    Thorough(HashType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupOptions {
    lock_source: bool,
    backup_acl: bool,
    backup_other_file: bool,
    advanced_file_attr: bool,
}

#[derive(Debug, Clone)]
pub struct BackupTask {
    pub uuid: Uuid,
    pub state: BackupState,
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub backup_type: BackupType,
    pub comparison_mode: Option<ComparisonMode>,
    pub options: BackupOptions,
    pub schedule: bool,
    pub last_run_time: Option<SystemTime>,
    pub next_run_time: Option<SystemTime>,
}
