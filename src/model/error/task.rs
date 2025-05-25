use crate::r#macro::loggable::loggable;

loggable! {
    TaskError {
        #[error("Task not found")]
        TaskNotFound: tracing::Level::ERROR,
        
        #[error("Failed to stop task")]
        StopTaskFailed: tracing::Level::ERROR,
    }
}
