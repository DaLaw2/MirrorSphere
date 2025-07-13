use crate::core::app_config::AppConfig;
use crate::core::backup_engine::{BackupEngine, RunBackupEngine};
use crate::core::database_manager::DatabaseManager;
use crate::core::event_bus::EventBus;
use crate::core::io_manager::IOManager;
use crate::core::progress_tracker::ProgressTracker;
use crate::model::error::Error;
use crate::model::log::system::SystemLog;
#[cfg(not(debug_assertions))]
use crate::platform::elevate::elevate;
use crate::utils::database_lock::DatabaseLock;
use crate::utils::logging::Logging;
use macros::log;
#[cfg(not(debug_assertions))]
use privilege::user::privileged;
#[cfg(not(debug_assertions))]
use std::process;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct System {
    pub event_bus: Arc<EventBus>,
    pub app_config: Arc<AppConfig>,
    pub io_manager: Arc<IOManager>,
    pub progress_tracker: Arc<ProgressTracker>,
    pub database_lock: DatabaseLock,
    pub database_manager: Arc<DatabaseManager>,
    pub backup_engine: Arc<BackupEngine>,
    pub backup_engine_shutdown: Option<oneshot::Sender<()>>,
}

impl System {
    pub async fn new(event_bus: Arc<EventBus>) -> Result<Self, Error> {
        Logging::initialize().await;
        log!(SystemLog::Initializing);
        #[cfg(not(debug_assertions))]
        if !privileged() {
            log!(SystemLog::ReRunAsAdmin);
            elevate()?;
            process::exit(0);
        }
        let app_config = Arc::new(AppConfig::new()?);
        let io_manager = Arc::new(IOManager::new(app_config.clone()));
        let progress_tracker = Arc::new(ProgressTracker::new(io_manager.clone()));
        let database_lock = DatabaseLock::acquire().await?;
        let database_manager = Arc::new(DatabaseManager::new().await?);
        let backup_engine = Arc::new(
            BackupEngine::new(
                app_config.clone(),
                event_bus.clone(),
                io_manager.clone(),
                progress_tracker.clone(),
            )
            .await,
        );
        log!(SystemLog::InitializeComplete);
        Ok(Self {
            event_bus,
            app_config,
            io_manager,
            progress_tracker,
            database_lock,
            database_manager,
            backup_engine,
            backup_engine_shutdown: None,
        })
    }

    pub async fn run(&mut self) {
        let shutdown = self.backup_engine.run().await;
        self.backup_engine_shutdown = Some(shutdown);
    }

    pub async fn terminate(&mut self) {
        log!(SystemLog::Terminating);
        if let Some(backup_engine_shutdown) = self.backup_engine_shutdown.take() {
            //todo Need add error handle
            let _ = backup_engine_shutdown.send(());
        }
        self.backup_engine.stop_all_executions().await;
        self.database_manager.close_connection().await;
        log!(SystemLog::TerminateComplete);
    }
}
