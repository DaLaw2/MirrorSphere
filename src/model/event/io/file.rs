use crate::interface::event_system::event::Event;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct CopyFileEvent {
    pub task_id: Uuid,
    pub source: PathBuf,
    pub destination: PathBuf,
}

impl Event for CopyFileEvent {}

#[derive(Clone)]
pub struct DeleteFileEvent {
    pub task_id: Uuid,
    pub path: PathBuf,
}

impl Event for DeleteFileEvent {}
