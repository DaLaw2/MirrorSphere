use tokio::fs::File;
use std::io;
use std::path::Path;
use fs4::tokio::AsyncFileExt;

#[derive(Debug)]
pub struct FileLock {
    file: File,
}

impl FileLock {
    pub async fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path).await?;
        file.try_lock_exclusive()?;
        Ok(Self { file })
    }

    pub async fn from_file(file: File) -> io::Result<Self> {
        file.try_lock_exclusive()?;
        Ok(Self { file })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
