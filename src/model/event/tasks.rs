use crate::interface::event::Event;
use crate::model::backup_execution::{BackupExecution, BackupState};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ExecutionAddRequest {
    pub execution: BackupExecution,
}
impl Event for ExecutionAddRequest {}

#[derive(Clone, Debug)]
pub struct ExecutionRemoveRequest {
    pub execution_id: Uuid,
}
impl Event for ExecutionRemoveRequest {}

#[derive(Clone, Debug)]
pub struct ExecutionStartRequest {
    pub execution_id: Uuid,
}
impl Event for ExecutionStartRequest {}

#[derive(Clone, Debug)]
pub struct ExecutionSuspendRequest {
    pub execution_id: Uuid,
}
impl Event for ExecutionSuspendRequest {}

#[derive(Clone, Debug)]
pub struct ExecutionResumeRequested {
    pub execution_id: Uuid,
}
impl Event for ExecutionResumeRequested {}

#[derive(Clone, Debug)]
pub struct ExecutionStateChanged {
    pub execution_id: Uuid,
    pub new_state: BackupState,
}
impl Event for ExecutionStateChanged {}

#[derive(Clone, Debug)]
pub struct ExecutionProgress {
    pub task_id: Uuid,
    pub processed_files: usize,
    pub error_count: usize,
}
impl Event for ExecutionProgress {}
