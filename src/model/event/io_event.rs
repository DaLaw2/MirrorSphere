use std::path::PathBuf;
use uuid::Uuid;
use crate::interface::event_system::event::Event;

#[derive(Clone)]
pub enum IOType {
    ListDirectory,
    CreateDirectory,
    CopyFile,
    DeleteFile,
    ChangeAttributes,
    ChangeAccessControlList,
}

#[derive(Clone)]
pub struct IOEvent {
    task_id: Uuid,
    io_type: IOType,
    source: Option<PathBuf>,
    destination: PathBuf,
}

impl Event for IOEvent {}
