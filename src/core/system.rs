use crate::core::app_config::AppConfig;
use crate::core::backup_engine::BackupEngine;
use crate::core::database_manager::DatabaseManager;
use crate::core::event_bus::EventBus;
use crate::core::gui_manager::GuiManager;
use crate::core::io_manager::IOManager;
use crate::core::progress_tracker::ProgressTracker;
use crate::core::schedule_manager::ScheduleManager;
use crate::interface::service_unit::ServiceUnit;
use crate::model::error::Error;
use crate::model::error::system::SystemError;
use crate::model::log::system::SystemLog;
#[cfg(not(debug_assertions))]
use crate::platform::elevate::elevate;
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
    pub event_bus: Arc<EventBus>,
    pub app_config: Arc<AppConfig>,
    pub io_manager: Arc<IOManager>,
    pub progress_tracker: Arc<ProgressTracker>,
    pub database_lock: DatabaseLock,
    pub database_manager: Arc<DatabaseManager>,
    pub gui_manager: Arc<GuiManager>,
    pub schedule_manager: Arc<ScheduleManager>,
    pub backup_engine: Arc<BackupEngine>,
    pub shutdowns: Vec<oneshot::Sender<()>>,
}

impl System {
    pub async fn new() -> Result<Self, Error> {
        Logging::initialize().await;
        log!(SystemLog::Initializing);
        #[cfg(not(debug_assertions))]
        if !privileged() {
            log!(SystemLog::ReRunAsAdmin);
            elevate()?;
            process::exit(0);
        }
        let app_config = Arc::new(AppConfig::new()?);
        let event_bus = Arc::new(EventBus::new());
        let io_manager = Arc::new(IOManager::new(app_config.clone()));
        let progress_tracker = Arc::new(ProgressTracker::new(io_manager.clone()));
        let database_lock = DatabaseLock::acquire().await?;
        let database_manager = Arc::new(DatabaseManager::new().await?);

        let schedule_manager = Arc::new(ScheduleManager::new(
            app_config.clone(),
            event_bus.clone(),
            database_manager.clone(),
        ));
        let backup_engine = Arc::new(
            BackupEngine::new(
                app_config.clone(),
                event_bus.clone(),
                io_manager.clone(),
                progress_tracker.clone(),
            )
            .await,
        );
        let gui_manager = Arc::new(GuiManager::new(
            event_bus.clone(),
            backup_engine.clone(),
            schedule_manager.clone(),
        ));
        log!(SystemLog::InitializeComplete);
        Ok(Self {
            event_bus,
            app_config,
            io_manager,
            progress_tracker,
            database_lock,
            database_manager,
            gui_manager,
            schedule_manager,
            backup_engine,
            shutdowns: Vec::new(),
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let gui_manager = self.gui_manager.clone();
        let schedule_manager_shutdown = self.schedule_manager.clone().run().await;
        let backup_engine_shutdown = self.backup_engine.clone().run().await;
        self.shutdowns.push(schedule_manager_shutdown);
        self.shutdowns.push(backup_engine_shutdown);
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
        log!(SystemLog::TerminateComplete);
    }
}
