use crate::model::error::Error;
use crate::model::error::misc::MiscError;
use eframe::egui::IconData;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

impl Assets {
    pub fn load_app_icon() -> Result<IconData, Error> {
        let icon_bytes = Assets::get("icon.ico")
            .ok_or(MiscError::AssertFileNotFound)?;

        let image = image::load_from_memory(&icon_bytes.data)
            .map_err(MiscError::DeserializeError)?
            .to_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();

        Ok(IconData {
            rgba,
            width,
            height,
        })
    }
}
