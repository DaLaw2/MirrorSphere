use crate::r#macro::loggable::loggable;

loggable! {
    SystemError {
        #[error("Rerun the program as administrator")]
        ReRunAsAdmin: tracing::Level::WARN,

        #[error("Unable to run as administrator")]
        RunAsAdminFailed: tracing::Level::ERROR,

        #[error("Failed to adjust token privileges")]
        AdjustTokenPrivilegesFailed: tracing::Level::ERROR,

        #[error("Failed to free object")]
        ObjectFreeFailed: tracing::Level::ERROR,

        #[error("Invalid configuration")]
        InvalidConfig: tracing::Level::ERROR,

        #[error("Configuration not found")]
        ConfigNotFound: tracing::Level::ERROR,

        #[error("Internal error")]
        InternalError: tracing::Level::ERROR,

        #[error("Unknown error")]
        ThreadPanic: tracing::Level::ERROR,

        #[error("Unknown error")]
        UnknownError: tracing::Level::ERROR,
    }
}
