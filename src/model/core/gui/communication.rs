use crate::interface::communication::event::Event;
use crate::model::error::Error;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct FolderProcess {
    uuid: Uuid,
    folder: PathBuf,
}

impl Event for FolderProcess {}

#[derive(Clone)]
pub struct ExecutionProgress {
    uuid: Uuid,
    processed_files: usize,
    error_count: usize,
}

impl Event for ExecutionProgress {}

#[derive(Clone)]
pub struct ExecutionErrors {
    uuid: Uuid,
    errors: Vec<Error>,
}

impl Event for ExecutionErrors {}
