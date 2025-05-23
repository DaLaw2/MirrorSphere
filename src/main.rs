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
    System::initialize().await;
    System::run().await;
    System::terminate().await;
}
