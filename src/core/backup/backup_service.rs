use crate::core::backup::backup_engine::BackupEngine;
use crate::core::backup::progress_tracker::ProgressTracker;
use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::communication_manager::CommunicationManager;
use crate::core::infrastructure::io_manager::IOManager;
use std::sync::Arc;

pub struct BackupService {
    backup_engine: Arc<BackupEngine>,
}

impl BackupService {
    pub async fn new(
        app_config: Arc<AppConfig>,
        io_manager: Arc<IOManager>,
        communication_manager: Arc<CommunicationManager>,
    ) -> Self {
        let progress_tracker = Arc::new(ProgressTracker::new(io_manager.clone()));
        let backup_engine = Arc::new(BackupEngine::new(
            app_config,
            io_manager,
            communication_manager,
            progress_tracker,
        ));
        Self { backup_engine }
    }

    pub async fn register_services(&self) {
        let backup_engine = self.backup_engine.clone();
        backup_engine.register_services().await;
    }
}
