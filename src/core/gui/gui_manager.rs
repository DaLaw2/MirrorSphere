use crate::core::infrastructure::actor_system::ActorSystem;
use crate::core::infrastructure::app_config::AppConfig;
use crate::interface::actor::actor::Actor;
use crate::interface::actor::message::Message;
use crate::model::core::gui::message::GuiMessage;
use crate::model::error::misc::MiscError;
use crate::model::error::Error;
use crate::ui::main_page::MainPage;
use crate::utils::assets::Assets;
use crate::utils::font;
use async_trait::async_trait;
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

    pub fn start(&self) -> Result<(), Error> {
        let app_config = self.app_config.clone();
        let actor_system = self.actor_system.clone();

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
                Ok(Box::new(MainPage::new(app_config, actor_system)))
            }),
        )
        .map_err(MiscError::UIPlatformError)?;

        Ok(())
    }
}

#[async_trait]
impl Actor for GuiManager {
    type Message = GuiMessage;

    async fn pre_start(&mut self) {}

    async fn post_stop(&mut self) {}

    async fn receive(
        &mut self,
        message: Self::Message,
    ) -> Result<<Self::Message as Message>::Response, Error> {
        //todo!()
        Ok(())
    }
}
