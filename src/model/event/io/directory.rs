use crate::interface::event_system::event::Event;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct ListDirectoryEvent {
    pub task_id: Uuid,
    pub path: PathBuf,
}

impl Event for ListDirectoryEvent {}

#[derive(Clone)]
pub struct CreateDirectoryEvent {
    pub task_id: Uuid,
    pub path: PathBuf,
}

impl Event for CreateDirectoryEvent {}

#[derive(Clone)]
pub struct DeleteDirectoryEvent {
    pub task_id: Uuid,
    pub path: PathBuf,
}

impl Event for DeleteDirectoryEvent {}
