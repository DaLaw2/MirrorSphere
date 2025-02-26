use crate::interface::event_system::event::Event;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub enum IOType {
    ListDirectory,
    CreateDirectory,
    DeleteDirectory,
    CopyFile,
    DeleteFile,
    ChangeAttributes,
    ChangeAccessControlList,
}

#[derive(Clone)]
pub struct IOEvent {
    pub task_id: Uuid,
    pub io_type: IOType,
    pub source: Option<PathBuf>,
    pub destination: PathBuf,
}

impl Event for IOEvent {}
