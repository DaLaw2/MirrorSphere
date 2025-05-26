use crate::r#macro::loggable::loggable;
use std::path::PathBuf;

loggable! {
    IOError {
        #[error("Semaphore has been closed")]
        SemaphoreClosed => tracing::Level::ERROR,

        #[error("Failed to create directory: {path}")]
        CreateDirectoryFailed { path: PathBuf } => tracing::Level::ERROR,

        #[error("Failed to read directory: {path}")]
        ReadDirectoryFailed { path: PathBuf } => tracing::Level::ERROR,

        #[error("Failed to read file: {path}")]
        ReadFileFailed { path: PathBuf } => tracing::Level::ERROR,

        #[error("Failed to copy file: From {src} To {dst}")]
        CopyFileFailed { src: PathBuf, dst: PathBuf } => tracing::Level::ERROR,

        #[error("Failed to delete directory: {path}")]
        DeleteDirectoryFailed { path: PathBuf } => tracing::Level::ERROR,

        #[error("Failed to delete file: {path}")]
        DeleteFileFailed { path: PathBuf } => tracing::Level::ERROR,

        #[error("Failed to get file metadata: {path}")]
        GetMetadataFailed { path: PathBuf } => tracing::Level::ERROR,

        #[error("Failed to set file metadata: {path}")]
        SetMetadataFailed { path: PathBuf } => tracing::Level::ERROR,
        
        #[error("Failed to lock file: {path}")]
        LockFileFailed { path: PathBuf } => tracing::Level::ERROR,
        
        #[error("Failed to unlock file: {path}")]     
        UnlockFileFailed { path: PathBuf } => tracing::Level::ERROR,
    }
}
