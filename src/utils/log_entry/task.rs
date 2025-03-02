use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskEntry {
    #[error("Task not found")]
    TaskNotFound,
}
