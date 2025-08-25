#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::core::system::System;

mod core;
mod interface;
mod model;
mod platform;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let system = System::new().await?;
    system.run().await?;
    system.shutdown().await;
    Ok(())
}
