use crate::core::app_config::AppConfig;
use crate::interface::file_system::FileSystemTrait;
use crate::platform::file_system::FileSystem;
use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use tokio::sync::Semaphore;

pub static IO_MANAGER: OnceLock<IOManager> = OnceLock::new();

pub struct IOManager {
    file_system: FileSystem,
}

impl IOManager {
    pub async fn initialize() {
        let config = AppConfig::fetch().await;
        let max_file_operations = config.max_file_operations;
        let semaphore = Arc::new(Semaphore::new(max_file_operations));
        let instance = IOManager {
            file_system: FileSystem::new(semaphore),
        };
        IO_MANAGER.get_or_init(|| instance);
    }

    pub fn instance() -> &'static IOManager {
        IO_MANAGER.get().unwrap()
    }
}

impl Deref for IOManager {
    type Target = FileSystem;

    fn deref(&self) -> &Self::Target {
        &self.file_system
    }
}
