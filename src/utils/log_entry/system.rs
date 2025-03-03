use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemEntry {
    #[error("Rerun as administrator")]
    ReRunAsAdmin,
    #[error("Unable to run as administrator")]
    RunAsAdminFailed,
    #[error("Online now")]
    Online,
    #[error("Initializing")]
    Initializing,
    #[error("Initialization completed")]
    InitializeComplete,
    #[error("Termination in process")]
    Terminating,
    #[error("Termination completed")]
    TerminateComplete,
    #[error("Invalid configuration")]
    InvalidConfig,
    #[error("Configuration not found")]
    ConfigNotFound,
    #[error("Internal error")]
    InternalError,
    #[error("Unknown error")]
    ThreadPanic,
}
