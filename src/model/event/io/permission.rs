use crate::interface::event_system::event::Event;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct GetPermissionEvent {
    pub task_id: Uuid,
    pub path: PathBuf,
}

impl Event for GetPermissionEvent {}

#[derive(Clone)]
pub struct SetPermissionEvent {
    pub task_id: Uuid,
    pub path: PathBuf,
}

impl Event for SetPermissionEvent {}
