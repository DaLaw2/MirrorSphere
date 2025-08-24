use crate::core::backup::backup_engine::BackupEngine;
use crate::core::backup::progress_tracker::ProgressTracker;
use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::io_manager::IOManager;
use crate::interface::core::service::Service;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;

pub struct BackupService {
    backup_engine: Arc<BackupEngine>,
}

impl BackupService {
    pub async fn init(app_config: Arc<AppConfig>, io_manager: Arc<IOManager>) {
        let progress_tracker = Arc::new(ProgressTracker::new(io_manager.clone()));
        let backup_engine = Arc::new(BackupEngine::new(app_config, io_manager, progress_tracker));
        let backup_service = Self { backup_engine };
    }
}

#[async_trait]
impl Service for BackupService {
    async fn run_message_loop(self: Arc<Self>, shutdown_rx: Receiver<()>) {
        todo!()
    }
}
