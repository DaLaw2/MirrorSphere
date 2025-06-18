use crate::core::app_config::AppConfig;
use crate::core::backup_engine::BackupEngine;
use crate::core::database_manager::DatabaseManager;
use crate::core::io_manager::IOManager;
use crate::model::error::system::SystemError;
use crate::model::log::system::SystemLog;
use crate::platform::elevate::elevate;
use crate::utils::logging::Logging;
use privilege::user::privileged;
use std::process;
use std::sync::Arc;
use crate::model::error::Error;

pub struct System {
    pub app_config: Arc<AppConfig>,
    pub backup_engine: Arc<BackupEngine>,
    pub database_manager: Arc<DatabaseManager>,
    pub io_manager: Arc<IOManager>,
}

impl System {
    pub async fn new() -> Result<Self, Error> {
        Logging::initialize().await;
        SystemLog::Initializing.log();
        if !privileged() {
            SystemLog::ReRunAsAdmin.log();
            elevate()
                .map_err(|_| SystemError::RunAsAdminFailed)?;
            process::exit(0);
        }
        let app_config = Arc::new(AppConfig::new().await?);
        let backup_engine = Arc::new(BackupEngine::initialize().await);
        let io_manager = Arc::new(IOManager::new(app_config.clone()).await);
        let database_manager = Arc::new(DatabaseManager::new().await);
        SystemLog::InitializeComplete.log();
        Ok(System {
            app_config,
            backup_engine,
            database_manager,
            io_manager,
        })
    }

    pub async fn run(&self) {
        SystemLog::Online.log();
        unimplemented!();
    }

    pub async fn terminate(&self) {
        SystemLog::Terminating.log();
        BackupEngine::terminate().await;
        DatabaseManager::terminate().await;
        SystemLog::TerminateComplete.log();
    }
}
