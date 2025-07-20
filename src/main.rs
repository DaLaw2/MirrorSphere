use crate::core::system::System;

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
