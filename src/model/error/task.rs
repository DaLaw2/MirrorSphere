use macros::traceable;

traceable! {
    TaskError {
        #[no_source]
        #[error("Illegal task state")]
        IllegalExecutionState => tracing::Level::ERROR,

        #[no_source]
        #[error("Task not found")]
        ExecutionNotFound => tracing::Level::ERROR,

        #[error("Failed to stop task")]
        StopExecutionFailed => tracing::Level::ERROR,


    }
}
