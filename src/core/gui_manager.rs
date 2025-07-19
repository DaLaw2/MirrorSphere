use crate::core::event_bus::EventBus;
use crate::model::error::misc::MiscError;
use crate::model::error::Error;
use crate::ui::main_page::MainPage;
use eframe::egui;
use std::sync::Arc;

pub struct GuiManager {
    event_bus: Arc<EventBus>
}

impl GuiManager {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus
        }
    }

    pub fn start(&self) -> Result<(), Error> {
        let event_bus = self.event_bus.clone();

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_title("MirrorSphere"),
            ..Default::default()
        };

        eframe::run_native(
            "MirrorSphere",
            options,
            Box::new(|_cc| Ok(Box::new(MainPage::new(event_bus)))),
        )
        .map_err(|err| MiscError::UIPlatformError(err))?;

        Ok(())
    }
}
