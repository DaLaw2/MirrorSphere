use crate::interface::event::Event;
use crate::model::error::Error;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct BackupError {
    pub task_id: Uuid,
    pub errors: Vec<Error>,
}

impl Event for BackupError {}

impl BackupError {
    pub fn new(task_id: Uuid, errors: Vec<Error>) -> Self {
        Self { task_id, errors }
    }
}
