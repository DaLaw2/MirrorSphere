use crate::interface::communication::event::Event;
use crate::model::error::Error;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct FolderProcess {
    pub uuid: Uuid,
    pub folder: PathBuf,
}

impl Event for FolderProcess {}

#[derive(Clone)]
pub struct ExecutionProgress {
    pub uuid: Uuid,
    pub processed_files: usize,
    pub error_count: usize,
}

impl Event for ExecutionProgress {}

#[derive(Clone)]
pub struct ExecutionErrors {
    pub uuid: Uuid,
    pub errors: Vec<Error>,
}

impl Event for ExecutionErrors {}
