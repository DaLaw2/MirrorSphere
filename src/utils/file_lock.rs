use crate::model::error::io::IOError;
use crate::model::error::Error;
use fs4::tokio::AsyncFileExt;
use macros::log;
use std::path::PathBuf;
use tokio::fs::File;

#[derive(Debug)]
pub struct FileLock {
    file: File,
    path: PathBuf,
}

impl FileLock {
    pub async fn new(path: &PathBuf) -> Result<Self, Error> {
        let file = File::open(path)
            .await
            .map_err(|err| IOError::ReadFileFailed(path.clone(), err))?;
        file.try_lock_exclusive()
            .map_err(|err| IOError::LockFileFailed(path.clone(), err))?;
        Ok(Self {
            file,
            path: path.clone(),
        })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let path = self.path.clone();
        if let Err(err) = self.file.unlock() {
            log!(IOError::UnlockFileFailed(path, err));
        }
    }
}
