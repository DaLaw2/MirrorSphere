use crate::core::app_config::AppConfig;
use crate::interface::file_system::FileSystemTrait;
use crate::platform::file_system::FileSystem;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct IOManager {
    file_system: FileSystem,
}

impl IOManager {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let max_file_operations = config.max_file_operations;
        let semaphore = Arc::new(Semaphore::new(max_file_operations));
        Self {
            file_system: FileSystem::new(semaphore),
        }
    }

    pub fn terminate(&self) {
        self.file_system.semaphore().close();
    }
}

impl Deref for IOManager {
    type Target = FileSystem;

    fn deref(&self) -> &Self::Target {
        &self.file_system
    }
}
