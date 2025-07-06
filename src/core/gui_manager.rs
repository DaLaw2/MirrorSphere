use crate::core::event_bus::EventBus;
use crate::model::error::misc::MiscError;
use crate::model::error::Error;
use crate::ui::backup_app::BackupApp;
use eframe::egui;
use std::sync::Arc;

pub struct GuiManager;

impl GuiManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self, event_bus: Arc<EventBus>) -> Result<(), Error> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_title("MirrorSphere"),
            ..Default::default()
        };

        eframe::run_native(
            "MirrorSphere",
            options,
            Box::new(|_cc| Ok(Box::new(BackupApp::new(event_bus)))),
        )
        .map_err(|_| MiscError::UIPlatformError)?;

        Ok(())
    }
}
