use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::interface::file_system::FileSystemTrait;

pub struct FileSystem {
    semaphore: Arc<Semaphore>,
}

impl FileSystem {
    pub fn test(&self) {

    }
}

impl FileSystemTrait for FileSystem {
    fn new(semaphore: Arc<Semaphore>) -> Self {
        FileSystem { semaphore }
    }

    fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }
}
