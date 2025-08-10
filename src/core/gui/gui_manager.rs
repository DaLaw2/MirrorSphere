use crate::core::gui::gui_message_handler::GuiMessageHandler;
use crate::core::infrastructure::actor_system::ActorSystem;
use crate::core::infrastructure::app_config::AppConfig;
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
    actor_system: Arc<ActorSystem>,
}

impl GuiManager {
    pub fn new(app_config: Arc<AppConfig>, actor_system: Arc<ActorSystem>) -> Self {
        Self {
            app_config,
            actor_system,
        }
    }

    pub async fn start(&self) -> Result<(), Error> {
        let app_config = self.app_config.clone();
        let actor_system = self.actor_system.clone();

        let mut handler = GuiMessageHandler::new();
        let message_rx = handler.subscribe();
        actor_system.spawn(handler).await;

        let execution_page =
            ExecutionPage::new(app_config.clone(), actor_system.clone(), message_rx);
        let schedule_page = SchedulePage::new(app_config, actor_system)?;
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
