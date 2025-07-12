use macros::traceable;

traceable! {
    TaskError {
        #[no_source]
        #[error("Illegal task state")]
        IllegalTaskState => tracing::Level::ERROR,

        #[no_source]
        #[error("Task not found")]
        TaskNotFound => tracing::Level::ERROR,

        #[error("Failed to stop task")]
        StopTaskFailed => tracing::Level::ERROR,
    }
}
