use crate::r#macro::loggable::loggable;

loggable! {
    IOError {
        #[error("Semaphore has been closed")]
        SemaphoreClosed: tracing::Level::ERROR,

        #[error("Failed to create directory")]
        CreateDirectoryFailed: tracing::Level::ERROR,

        #[error("Failed to read directory")]
        ReadDirectoryFailed: tracing::Level::ERROR,

        #[error("Failed to read file")]
        ReadFileFailed: tracing::Level::ERROR,

        #[error("Failed to copy file")]
        CopyFileFailed: tracing::Level::ERROR,

        #[error("Failed to delete directory")]
        DeleteDirectoryFailed: tracing::Level::ERROR,

        #[error("Failed to delete file")]
        DeleteFileFailed: tracing::Level::ERROR,

        #[error("Failed to get file metadata")]
        GetMetadataFailed: tracing::Level::ERROR,

        #[error("Failed to set file metadata")]
        SetMetadataFailed: tracing::Level::ERROR,
    }
}
