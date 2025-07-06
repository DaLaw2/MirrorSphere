use crate::interface::event::Event;
use crate::model::task::{BackupState, BackupTask};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TaskAddRequested {
    pub task: BackupTask,
}
impl Event for TaskAddRequested {}

#[derive(Clone, Debug)]
pub struct TaskRemoveRequested {
    pub task_id: Uuid,
}
impl Event for TaskRemoveRequested {}

#[derive(Clone, Debug)]
pub struct TaskStartRequested {
    pub task_id: Uuid,
}
impl Event for TaskStartRequested {}

#[derive(Clone, Debug)]
pub struct TaskSuspendRequested {
    pub task_id: Uuid,
}
impl Event for TaskSuspendRequested {}

#[derive(Clone, Debug)]
pub struct TaskResumeRequested {
    pub task_id: Uuid,
}
impl Event for TaskResumeRequested {}

#[derive(Clone, Debug)]
pub struct TaskStateChanged {
    pub task_id: Uuid,
    pub new_state: BackupState,
}
impl Event for TaskStateChanged {}

#[derive(Clone, Debug)]
pub struct TaskProgress {
    pub task_id: Uuid,
    pub processed_files: usize,
    pub error_count: usize,
}
impl Event for TaskProgress {}
