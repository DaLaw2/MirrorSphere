use crate::interface::event_system::event::Event;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct GetAttributesEvent {
    pub task_id: Uuid,
    pub path: PathBuf,
}

impl Event for GetAttributesEvent {}

#[derive(Clone)]
pub struct ChangeAttributesEvent {
    pub task_id: Uuid,
    pub path: PathBuf,
}

impl Event for ChangeAttributesEvent {}
