use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupState {
    Running,
    Pending,
    Suspended,
    Completed,
    Failed,
    Canceled,
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
    pub mirror: bool,    
    pub lock_source: bool,
    pub backup_permission: bool,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone)]
pub struct BackupExecution {
    pub uuid: Uuid,
    pub state: BackupState,
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub backup_type: BackupType,
    pub comparison_mode: Option<ComparisonMode>,
    pub options: BackupOptions,
}
