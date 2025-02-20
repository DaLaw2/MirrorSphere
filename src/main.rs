#![feature(file_lock)]
#![feature(trait_alias)]

use crate::core::system::System;

mod core;
mod interface;
mod model;
mod platform;
mod ui;
mod utils;

#[tokio::main]
async fn main() {
    System::initialize().await;
    System::run().await;
    System::terminate().await;
}
