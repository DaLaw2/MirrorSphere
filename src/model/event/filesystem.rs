use crate::interface::event::Event;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct FolderProcessing {
    pub execution_id: Uuid,
    pub current_folder: PathBuf,
}
impl Event for FolderProcessing {}
