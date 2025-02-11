use crate::model::backup_task::{BackupOptions, ComparisonMode};
use crate::model::diff_entry::DiffEntry;
use std::path::PathBuf;

pub struct Comparator;

impl Comparator {
    pub fn compare_directory(
        source: PathBuf,
        destination: PathBuf,
        comparison_mode: ComparisonMode,
        backup_options: BackupOptions,
    ) -> Vec<DiffEntry> {

    }
}
