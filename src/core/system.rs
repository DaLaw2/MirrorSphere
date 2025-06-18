use crate::core::app_config::AppConfig;
use crate::core::backup_engine::BackupEngine;
use crate::core::database_manager::DatabaseManager;
use crate::core::io_manager::IOManager;
use crate::core::progress_tracker::ProgressTracker;
use crate::model::error::system::SystemError;
use crate::model::error::Error;
use crate::model::log::system::SystemLog;
use crate::platform::elevate::elevate;
use crate::utils::logging::Logging;
use privilege::user::privileged;
use std::process;
use std::sync::Arc;

pub struct System {
    pub app_config: Arc<AppConfig>,
    pub io_manager: Arc<IOManager>,
    pub progress_tracker: Arc<ProgressTracker>,
    pub database_manager: Arc<DatabaseManager>,
    pub backup_engine: Arc<BackupEngine>,
}

impl System {
    pub async fn new() -> Result<Self, Error> {
        Logging::initialize().await;
        SystemLog::Initializing.log();
        if !privileged() {
            SystemLog::ReRunAsAdmin.log();
            elevate().map_err(|_| SystemError::RunAsAdminFailed)?;
            process::exit(0);
        }
        let app_config = Arc::new(AppConfig::new().await?);
        let io_manager = Arc::new(IOManager::new(app_config.clone()).await);
        let progress_tracker = Arc::new(ProgressTracker::new(io_manager.clone()));
        let database_manager = Arc::new(DatabaseManager::new().await?);
        let backup_engine = Arc::new(
            BackupEngine::new(
                app_config.clone(),
                io_manager.clone(),
                progress_tracker.clone(),
            )
            .await,
        );
        SystemLog::InitializeComplete.log();
        Ok(System {
            app_config,
            io_manager,
            progress_tracker,
            database_manager,
            backup_engine,
        })
    }

    pub async fn run(&self) {
        SystemLog::Online.log();
        unimplemented!();
    }

    pub async fn terminate(&self) {
        SystemLog::Terminating.log();
        self.backup_engine.terminate().await;
        self.database_manager.terminate().await;
        SystemLog::TerminateComplete.log();
    }
}
