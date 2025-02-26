use crate::interface::file_system::FileSystemTrait;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct FileSystem {
    semaphore: Arc<Semaphore>,
}

#[async_trait]
impl FileSystemTrait for FileSystem {
    fn new(semaphore: Arc<Semaphore>) -> Self {
        FileSystem { semaphore }
    }

    fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }
}
