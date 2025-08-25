use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::communication_manager::CommunicationManager;
use crate::model::error::misc::MiscError;
use crate::model::error::Error;
use crate::ui::execution_page::ExecutionPage;
use crate::ui::main_page::MainPage;
use crate::ui::schedule_page::SchedulePage;
use crate::utils::assets::Assets;
use crate::utils::font;
use eframe::egui;
use std::sync::Arc;

pub struct GuiManager {
    app_config: Arc<AppConfig>,
    communication_manager: Arc<CommunicationManager>,
}

impl GuiManager {
    pub fn new(
        app_config: Arc<AppConfig>,
        communication_manager: Arc<CommunicationManager>,
    ) -> Self {
        Self {
            app_config,
            communication_manager,
        }
    }

    pub async fn start(&self) -> Result<(), Error> {
        let app_config = self.app_config.clone();
        let communication_manager = self.communication_manager.clone();

        let execution_page = ExecutionPage::new(app_config.clone(), communication_manager.clone())?;
        let schedule_page = SchedulePage::new(app_config, communication_manager)?;
        let main_page = MainPage::new(execution_page, schedule_page);

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
                Ok(Box::new(main_page))
            }),
        )
        .map_err(MiscError::UIPlatformError)?;

        Ok(())
    }
}
