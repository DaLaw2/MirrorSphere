use std::sync::Arc;
use eframe::egui;
use font_kit::source::SystemSource;
use font_kit::family_name::FamilyName;
use font_kit::properties::{Properties, Weight, Style, Stretch};

pub fn setup_system_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let system_source = SystemSource::new();

    if let Ok(font_handle) = system_source.select_best_match(
        &[FamilyName::Title("Times New Roman".to_string())],
        &Properties {
            weight: Weight::NORMAL,
            style: Style::Normal,
            stretch: Stretch::NORMAL,
        }
    ) {
        if let Ok(font) = font_handle.load() {
            if let Some(font_data) = font.copy_font_data() {
                fonts.font_data.insert(
                    "times_new_roman".to_owned(),
                    Arc::from(egui::FontData::from_owned(font_data.to_vec()))
                );
            }
        }
    }

    if let Ok(font_handle) = system_source.select_best_match(
        &[
            FamilyName::Title("DFKai-SB".to_string()),
            FamilyName::Title("BiauKai".to_string()),
        ],
        &Properties::default()
    ) {
        if let Ok(font) = font_handle.load() {
            if let Some(font_data) = font.copy_font_data() {
                fonts.font_data.insert(
                    "kaiti".to_owned(),
                    Arc::from(egui::FontData::from_owned(font_data.to_vec()))
                );
            }
        }
    }

    if let Ok(emoji_handle) = system_source.select_best_match(
        &[
            FamilyName::Title("Segoe UI Symbol".to_string()),
            FamilyName::Title("Segoe UI Emoji".to_string()),
        ],
        &Properties::default()
    ) {
        if let Ok(font) = emoji_handle.load() {
            if let Some(font_data) = font.copy_font_data() {
                fonts.font_data.insert(
                    "system_emoji".to_owned(),
                    Arc::from(egui::FontData::from_owned(font_data.to_vec()))
                );
            }
        }
    }

    fonts.families.get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "times_new_roman".to_owned());
    fonts.families.get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(1, "kaiti".to_owned());
    fonts.families.get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(2, "system_emoji".to_owned());
    fonts.families.get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "times_new_roman".to_owned());
    fonts.families.get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(1, "kaiti".to_owned());

    ctx.set_fonts(fonts);
}
