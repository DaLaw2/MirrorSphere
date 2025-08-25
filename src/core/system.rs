use crate::core::backup::backup_service::BackupService;
use crate::core::gui::gui_manager::GuiManager;
use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::communication_manager::CommunicationManager;
use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::core::infrastructure::io_manager::IOManager;
use crate::core::schedule::schedule_service::ScheduleService;
use crate::interface::core::runnable::Runnable;
use crate::model::error::Error;
use crate::model::log::system::SystemLog;
#[cfg(any(target_os = "windows", not(debug_assertions)))]
use crate::platform::elevate;
use crate::utils::logging::Logging;
use crossbeam_queue::SegQueue;
use macros::log;
#[cfg(not(debug_assertions))]
use privilege::user::privileged;
#[cfg(not(debug_assertions))]
use std::process;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct System {
    backup_service: Arc<BackupService>,
    schedule_service: Arc<ScheduleService>,
    gui_manager: Arc<GuiManager>,
    shutdowns: SegQueue<oneshot::Sender<()>>,
}

impl System {
    pub async fn new() -> Result<Self, Error> {
        let app_config = Arc::new(AppConfig::new()?);
        let io_manager = Arc::new(IOManager::new(app_config.clone()));
        let database_manager = Arc::new(DatabaseManager::new().await?);
        let communication_manager = Arc::new(CommunicationManager::new(app_config.clone()));
        let backup_service = Arc::new(
            BackupService::new(
                app_config.clone(),
                io_manager.clone(),
                communication_manager.clone(),
            )
            .await,
        );
        let schedule_service = Arc::new(
            ScheduleService::new(
                app_config.clone(),
                database_manager.clone(),
                communication_manager.clone(),
            )
            .await?,
        );
        let gui_manager = Arc::new(GuiManager::new(app_config, communication_manager));
        let system = Self {
            backup_service,
            schedule_service,
            gui_manager,
            shutdowns: SegQueue::new(),
        };
        Ok(system)
    }

    pub async fn run(&self) -> Result<(), Error> {
        Logging::initialize().await;
        log!(SystemLog::Initializing);
        Self::elevate_privileges()?;
        let backup_service = self.backup_service.clone();
        let schedule_service = self.schedule_service.clone();
        let gui_manager = self.gui_manager.clone();
        backup_service.register_services().await;
        schedule_service.register_services().await;
        let schedule_service_shutdown = schedule_service.run().await;
        self.shutdowns.push(schedule_service_shutdown);
        log!(SystemLog::InitializeComplete);
        gui_manager.start().await
    }

    pub async fn shutdown(&self) {
        self.backup_service.shutdown().await;
        while let Some(shutdown) = self.shutdowns.pop() {
            let _ = shutdown.send(());
        }
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
