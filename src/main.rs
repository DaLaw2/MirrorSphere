use crate::core::event_bus::EventBus;
use crate::core::system::System;
use std::sync::Arc;
use crate::core::gui_manager::GuiManager;

mod core;
mod interface;
mod model;
mod platform;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_bus = Arc::new(EventBus::new());
    let mut system = System::new(event_bus.clone()).await?;
    let gui_manager = GuiManager::new();
    system.run().await;
    gui_manager.start(event_bus.clone())?;
    system.terminate().await;
    Ok(())
}
