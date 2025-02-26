use crate::interface::file_system::FileSystemTrait;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct FileSystem {
    semaphore: Arc<Semaphore>,
}

impl FileSystemTrait for FileSystem {
    fn new(semaphore: Arc<Semaphore>) -> Self {
        FileSystem { semaphore }
    }

    fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }
}
