use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupState {
    Running,
    Suspended,
    Stopped,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupType {
    Full,
    Incremental,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    MD5,
    SHA3,
    SHA256,
    BLAKE2B,
    BLAKE2S,
    BLAKE3,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonMode {
    // Compare size and modify time
    Standard,
    // Standard + regular file attr
    Advanced,
    // Advanced + compare file checksum
    Thorough(HashType),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BackupOptions {
    pub lock_source: bool,
    pub backup_permission: bool,
    pub follow_symlinks: bool,
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

impl BackupTask {
    pub fn to_worker_task(&self) -> WorkerTask {
        WorkerTask {
            uuid: self.uuid,
            source_path: self.source_path.clone(),
            destination_path: self.destination_path.clone(),
            backup_type: self.backup_type,
            comparison_mode: self.comparison_mode,
            options: self.options,       
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkerTask {
    pub uuid: Uuid,
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub backup_type: BackupType,
    pub comparison_mode: Option<ComparisonMode>,
    pub options: BackupOptions,
}
