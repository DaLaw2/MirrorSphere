use crate::core::system::System;

mod core;
mod model;
mod ui;
mod utils;

#[tokio::main]
async fn main() {
    System::initialize().await;
    System::run().await;
    System::terminate().await;
}
