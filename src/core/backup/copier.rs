use std::path::PathBuf;
use crate::model::backup_task::BackupOptions;
use crate::model::diff_entry::DiffEntry;

pub struct Copier;

impl Copier {
    pub fn direct_copy(source: PathBuf, destination: PathBuf, options: BackupOptions) {

    }

    pub fn diff_copy(diff_entry: Vec<DiffEntry>, options: BackupOptions) {

    }
}
