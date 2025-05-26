use crate::interface::event_system::actor::Actor;
use std::path::PathBuf;
use uuid::Uuid;

pub struct ProgressTracker {}

impl ProgressTracker {
    pub async fn save_task(uuid: Uuid, queue: Vec<PathBuf>, errors: Vec<anyhow::Error>) {
        todo!()
    }

    pub async fn resume_task(uuid: Uuid) -> (Vec<PathBuf>, Vec<anyhow::Error>) {
        todo!()
    }
}

impl Actor for ProgressTracker {}
