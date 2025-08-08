use crate::core::infrastructure::app_config::AppConfig;
use crate::core::backup::backup_engine::BackupEngine;
use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::core::event_bus::EventBus;
use crate::core::gui::gui_manager::GuiManager;
use crate::core::infrastructure::io_manager::IOManager;
use crate::core::backup::progress_tracker::ProgressTracker;
use crate::core::schedule::schedule_manager::ScheduleManager;
use crate::interface::service_unit::ServiceUnit;
use crate::model::error::Error;
use crate::model::error::system::SystemError;
use crate::model::log::system::SystemLog;
#[cfg(any(target_os = "windows", not(debug_assertions)))]
use crate::platform::elevate;
use crate::utils::database_lock::DatabaseLock;
use crate::utils::logging::Logging;
use macros::log;
#[cfg(not(debug_assertions))]
use privilege::user::privileged;
use std::mem;
#[cfg(not(debug_assertions))]
use std::process;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct System {
    pub io_manager: Arc<IOManager>,
    pub database_manager: Arc<DatabaseManager>,
    pub backup_engine: Arc<BackupEngine>,
    pub schedule_manager: Arc<ScheduleManager>,
    pub gui_manager: Arc<GuiManager>,
    pub _database_lock: DatabaseLock,
    pub shutdowns: Vec<oneshot::Sender<()>>,
}

impl System {
    pub async fn new() -> Result<Self, Error> {
        Logging::initialize().await;
        log!(SystemLog::Initializing);
        Self::elevate_privileges()?;
        let app_config = Arc::new(AppConfig::new()?);
        let event_bus = Arc::new(EventBus::new());
        let io_manager = Arc::new(IOManager::new(app_config.clone()));
        let _database_lock = DatabaseLock::acquire().await?;
        let database_manager = Arc::new(DatabaseManager::new().await?);
        let progress_tracker = Arc::new(ProgressTracker::new(io_manager.clone()));
        let backup_engine = Arc::new(BackupEngine::new(
            app_config.clone(),
            event_bus.clone(),
            io_manager.clone(),
            progress_tracker.clone(),
        ));
        let schedule_manager = Arc::new(ScheduleManager::new(
            app_config.clone(),
            event_bus.clone(),
            database_manager.clone(),
        ));
        let gui_manager = Arc::new(GuiManager::new(
            app_config.clone(),
            event_bus.clone(),
            backup_engine.clone(),
            schedule_manager.clone(),
        ));
        log!(SystemLog::InitializeComplete);
        Ok(Self {
            io_manager,
            database_manager,
            backup_engine,
            schedule_manager,
            gui_manager,
            _database_lock,
            shutdowns: Vec::new(),
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let gui_manager = self.gui_manager.clone();
        let backup_engine_shutdown = self.backup_engine.clone().run().await;
        let schedule_manager_shutdown = self.schedule_manager.clone().run().await;
        self.shutdowns.push(backup_engine_shutdown);
        self.shutdowns.push(schedule_manager_shutdown);
        gui_manager.start()
    }

    pub async fn terminate(&mut self) {
        log!(SystemLog::Terminating);
        let shutdowns = mem::take(&mut self.shutdowns);
        for shutdown in shutdowns {
            if shutdown.send(()).is_err() {
                log!(SystemError::ShutdownSignalFailed);
            }
        }
        self.backup_engine.stop_all_executions().await;
        self.database_manager.close_connection().await;
        self.io_manager.terminate();
        log!(SystemLog::TerminateComplete);
    }

    fn elevate_privileges() -> Result<(), Error> {
        #[cfg(not(debug_assertions))]
        if !privileged() {
            log!(SystemLog::ReRunAsAdmin);
            elevate::elevate()?;
            process::exit(0);
        }
        #[cfg(target_os = "windows")]
        elevate::adjust_token_privileges()?;
        Ok(())
    }
}
