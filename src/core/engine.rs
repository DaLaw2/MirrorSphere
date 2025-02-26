use std::sync::OnceLock;
use dashmap::DashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::model::backup_task::BackupTask;

pub static ENGINE: OnceLock<RwLock<Engine>> = OnceLock::new();

#[derive(Debug)]
pub struct Engine {
}

impl Engine {
    pub async fn initialize() {
        ENGINE.set(RwLock::new(Engine {})).unwrap();
    }

    pub async fn run() {

    }

    pub async fn terminate() {

    }

    pub async fn create_task() {

    }

    pub async fn start_task() {

    }

    pub async fn suspend_task() {

    }

    pub async fn resume_task() {

    }
}
