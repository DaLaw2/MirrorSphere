use crate::core::backup_engine::BackupEngine;
use crate::core::event_bus::EventBus;
use crate::core::schedule_manager::ScheduleManager;
use crate::model::error::Error;
use crate::model::error::misc::MiscError;
use crate::ui::main_page::MainPage;
use eframe::egui;
use std::sync::Arc;
use crate::core::app_config::AppConfig;

pub struct GuiManager {
    config: Arc<AppConfig>,
    event_bus: Arc<EventBus>,
    backup_engine: Arc<BackupEngine>,
    schedule_manager: Arc<ScheduleManager>,
}

impl GuiManager {
    pub fn new(
        config: Arc<AppConfig>,
        event_bus: Arc<EventBus>,
        backup_engine: Arc<BackupEngine>,
        schedule_manager: Arc<ScheduleManager>,
    ) -> Self {
        Self {
            config,
            event_bus,
            backup_engine,
            schedule_manager,
        }
    }

    pub fn start(&self) -> Result<(), Error> {
        let config = self.config.clone();
        let event_bus = self.event_bus.clone();
        let backup_engine = self.backup_engine.clone();
        let schedule_manager = self.schedule_manager.clone();

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_title("MirrorSphere"),
            ..Default::default()
        };

        eframe::run_native(
            "MirrorSphere",
            options,
            Box::new(|_| {
                Ok(Box::new(MainPage::new(
                    config,
                    event_bus,
                    backup_engine,
                    schedule_manager,
                )))
            }),
        )
        .map_err(|err| MiscError::UIPlatformError(err))?;

        Ok(())
    }
}
