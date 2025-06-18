use crate::core::system::System;

mod core;
mod interface;
mod model;
mod platform;
mod ui;
mod utils;
mod r#macro;

#[tokio::main]
async fn main() {
    let system = System::new().await.unwrap();
    system.run().await;
    system.terminate().await;
}
