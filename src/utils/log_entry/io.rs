use thiserror::Error;

#[derive(Error, Debug)]
pub enum IOEntry {
    #[error("Semaphore has been closed")]
    SemaphoreClosed,
    #[error("Create directory failed")]
    CreateDirectoryFailed,
    #[error("Read directory failed")]
    ReadDirectoryFailed,
    #[error("Read file failed")]
    ReadFileFailed,
    #[error("Copy file failed")]
    CopyFileFailed,
    #[error("Delete directory failed")]
    DeleteDirectoryFailed,
    #[error("Delete file failed")]
    DeleteFileFailed,
    #[error("Get file metadata failed")]
    GetMetadataFailed,
    #[error("Set file metadata failed")]
    SetMetadataFailed,
}
