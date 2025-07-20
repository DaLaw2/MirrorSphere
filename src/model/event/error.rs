use crate::interface::event::Event;
use crate::model::error::Error;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct BackupError {
    pub task_id: Uuid,
    pub error: Error,
}
impl Event for BackupError {}

//todo Need add global error event
