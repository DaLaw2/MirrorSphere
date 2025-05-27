use crate::r#macro::loggable::loggable;

loggable! {
    MiscError {
        #[error("Failed to free object")]
        ObjectFreeFailed => tracing::Level::ERROR,

        #[error("Unexpected error type")]
        UnexpectedErrorType => tracing::Level::ERROR,
        
        #[error("Failed to decode bincode")]
        BincodeDecodeError => tracing::Level::ERROR,
    }
}
