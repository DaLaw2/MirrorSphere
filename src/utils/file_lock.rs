use crate::model::error::io::IOError;
use crate::model::error::Error;
use fs4::tokio::AsyncFileExt;
use std::path::PathBuf;
use tokio::fs::File;
use crate::log;

#[derive(Debug)]
pub struct FileLock {
    file: File,
    path: PathBuf,
}

impl FileLock {
    pub async fn new(path: &PathBuf) -> Result<Self, Error> {
        let file = File::open(path)
            .await
            .map_err(|_| IOError::ReadFileFailed { path: path.clone() })?;
        file.try_lock_exclusive()
            .map_err(|_| IOError::LockFileFailed { path: path.clone() })?;
        Ok(Self {
            file,
            path: path.clone(),
        })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let path = self.path.clone();
        if let Err(_) = self.file.unlock() {
            log!(IOError::UnlockFileFailed { path });
        }
    }
}
