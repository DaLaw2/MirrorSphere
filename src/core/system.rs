use crate::core::backup::backup_service::BackupService;
use crate::core::gui::gui_manager::GuiManager;
use crate::core::infrastructure::actor_system::ActorSystem;
use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::core::infrastructure::io_manager::IOManager;
use crate::core::schedule::schedule_service::ScheduleService;
use crate::model::error::Error;
use crate::model::log::system::SystemLog;
#[cfg(any(target_os = "windows", not(debug_assertions)))]
use crate::platform::elevate;
use crate::utils::logging::Logging;
use macros::log;
#[cfg(not(debug_assertions))]
use privilege::user::privileged;
#[cfg(not(debug_assertions))]
use std::process;
use std::sync::Arc;

pub struct System;

impl System {
    pub async fn run() -> Result<(), Error> {
        Logging::initialize().await;
        log!(SystemLog::Initializing);
        Self::elevate_privileges()?;
        let app_config = Arc::new(AppConfig::new()?);
        let io_manager = Arc::new(IOManager::new(app_config.clone()));
        let database_manager = Arc::new(DatabaseManager::new().await?);
        let actor_system = Arc::new(ActorSystem::new());
        BackupService::init(app_config.clone(), io_manager.clone(), actor_system.clone()).await;
        ScheduleService::init(
            app_config.clone(),
            database_manager.clone(),
            actor_system.clone(),
        )
        .await?;
        let gui_manager = Arc::new(GuiManager::new(app_config.clone(), actor_system.clone()));
        log!(SystemLog::InitializeComplete);
        gui_manager.start()
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
