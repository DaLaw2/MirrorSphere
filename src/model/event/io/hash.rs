use crate::interface::event_system::event::Event;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct CalculateHashEvent {
    pub task_id: Uuid,
    pub path: PathBuf,
}

impl Event for CalculateHashEvent {}
