use crate::core::infrastructure::app_config::AppConfig;
use crate::core::backup::backup_engine::BackupEngine;
use crate::core::schedule::schedule_manager::ScheduleManager;
use crate::model::error::Error;
use crate::model::error::misc::MiscError;
use crate::ui::main_page::MainPage;
use crate::utils::assets::Assets;
use crate::utils::font;
use eframe::egui;
use std::sync::Arc;

pub struct GuiManager {
    app_config: Arc<AppConfig>,
    event_bus: Arc<EventBus>,
    backup_engine: Arc<BackupEngine>,
    schedule_manager: Arc<ScheduleManager>,
}

impl GuiManager {
    pub fn new(
        app_config: Arc<AppConfig>,
        event_bus: Arc<EventBus>,
        backup_engine: Arc<BackupEngine>,
        schedule_manager: Arc<ScheduleManager>,
    ) -> Self {
        Self {
            app_config,
            event_bus,
            backup_engine,
            schedule_manager,
        }
    }

    pub fn start(&self) -> Result<(), Error> {
        let config = self.app_config.clone();
        let event_bus = self.event_bus.clone();
        let backup_engine = self.backup_engine.clone();
        let schedule_manager = self.schedule_manager.clone();

        let icon_data = Assets::load_app_icon()?;

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([960.0, 540.0])
                .with_title("MirrorSphere")
                .with_icon(icon_data),
            ..Default::default()
        };

        eframe::run_native(
            "MirrorSphere",
            options,
            Box::new(|cc| {
                font::setup_system_fonts(&cc.egui_ctx);
                Ok(Box::new(MainPage::new(
                    config,
                    event_bus,
                    backup_engine,
                    schedule_manager,
                )))
            }),
        )
        .map_err(MiscError::UIPlatformError)?;

        Ok(())
    }
}
