use std::path::PathBuf;

pub enum DiffType {
    Created,
    Modified,
    Deleted,
}

pub struct DiffEntry {
    pub diff_type: DiffType,
    pub source: Option<PathBuf>,
    pub destination: Option<PathBuf>,
}
