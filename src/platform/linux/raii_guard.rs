use crate::model::error::io::IOError;
use fs4::fs_std::FileExt;
use std::path::PathBuf;

pub struct FileLockGuard {
    path: PathBuf,
}

impl FileLockGuard {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        match std::fs::File::open(&self.path) {
            Ok(file) => {
                if FileExt::unlock(&file).is_err() {
                    IOError::UnlockFileFailed.log();
                }
            }
            Err(_) => IOError::ReadFileFailed.log(),
        }
    }
}
