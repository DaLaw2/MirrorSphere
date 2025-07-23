use macros::traceable;

traceable! {
    TaskError {
        #[no_source]
        #[error("Illegal run state")]
        IllegalRunState => tracing::Level::ERROR,

        #[no_source]
        #[error("Task not found")]
        ExecutionNotFound => tracing::Level::ERROR,

        #[error("Failed to stop task")]
        StopExecutionFailed => tracing::Level::ERROR,

        #[error("Failed to load schedule")]
        LoadScheduleFailed => tracing::Level::ERROR,

        #[error("Failed to enable schedule")]
        EnableScheduleFailed => tracing::Level::ERROR,

        #[error("Failed to pause schedule")]
        PauseScheduleFailed => tracing::Level::ERROR,

        #[error("Failed to disable schedule")]
        DisableScheduleFailed => tracing::Level::ERROR,

        #[error("Failed to resume schedule")]
        ResumeScheduleFailed => tracing::Level::ERROR,

        #[error("Failed to remove schedule")]
        RemoveScheduleFailed => tracing::Level::ERROR,
    }
}
