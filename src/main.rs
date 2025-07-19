use crate::core::event_bus::EventBus;
use crate::core::gui_manager::GuiManager;
use crate::core::system::System;
use std::sync::Arc;

mod core;
mod interface;
mod model;
mod platform;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::new().await?;
    system.run().await?;
    system.terminate().await;
    Ok(())
}
