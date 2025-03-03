use thiserror::Error;

#[derive(Error, Debug)]
pub enum IOEntry {
    #[error("Semaphore has been closed")]
    SemaphoreClosed,
    #[error("Read file failed")]
    ReadFileFailed,
    #[error("Get file metadata failed")]
    GetMetadataFailed,
}
