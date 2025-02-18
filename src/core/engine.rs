use std::sync::OnceLock;
use tokio::sync::RwLock;

pub static ENGINE: OnceLock<RwLock<Engine>> = OnceLock::new();

#[derive(Debug)]
pub struct Engine {}

impl Engine {
    pub async fn initialize() {
        ENGINE.set(RwLock::new(Engine {})).unwrap();
    }

    pub async fn terminate() {}
}
