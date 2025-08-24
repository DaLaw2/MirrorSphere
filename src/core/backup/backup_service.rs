use crate::core::backup::backup_engine::BackupEngine;
use crate::core::backup::progress_tracker::ProgressTracker;
use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::io_manager::IOManager;
use crate::interface::core::service::Service;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::sync::oneshot::Receiver;
use crate::interface::communication::command::CommandHandler;
use crate::interface::communication::query::QueryHandler;
use crate::interface::core::unit::Unit;
use crate::model::core::backup::communication::{BackupCommand, BackupQuery, BackupQueryResponse};
use crate::model::error::Error;

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
    async fn process_internal_command(self: Arc<Self>, shutdown_rx: Receiver<()>) {
        let mut backup_engine_rx = self.backup_engine.get_internal_channel();
        loop {
            select! {
                _ = &shutdown_rx => {
                    break
                }
                _ = backup_engine_rx.recv() => {
                    
                }
            }
        }
    }
}

#[async_trait]
impl CommandHandler<BackupCommand> for BackupService {
    async fn handle_command(&self, command: BackupCommand) -> Result<(), Error> {
        self.backup_engine.handle_command(command)
    }
}

#[async_trait]
impl QueryHandler<BackupQuery> for BackupService {
    async fn handle_query(&self, query: BackupQuery) -> Result<BackupQueryResponse, Error> {
        self.backup_engine.handle_query(query)
    }
}
