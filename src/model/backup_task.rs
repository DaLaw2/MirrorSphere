use crate::model::backup_options::BackupOptions;
use crate::model::backup_type::BackupType;
use crate::model::comparison_mode::ComparisonMode;
use std::path::PathBuf;

pub struct BackupTask {
    source_path: PathBuf,
    destination_path: PathBuf,
    backup_type: BackupType,
    comparison_mode: ComparisonMode,
    option: BackupOptions,
}
