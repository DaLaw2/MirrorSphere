use crate::interface::actor::message::Message;
use crate::model::error::Error;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub enum GuiMessage {
    FolderProcess {
        uuid: Uuid,
        folder: PathBuf,
    },
    ExecutionProgress {
        uuid: Uuid,
        processed_files: usize,
        error_count: usize,
    },
    ExecutionErrors {
        uuid: Uuid,
        errors: Vec<Error>,
    },
}

impl Message for GuiMessage {
    type Response = ();
}
