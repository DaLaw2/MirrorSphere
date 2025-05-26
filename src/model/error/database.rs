use crate::r#macro::loggable::loggable;

loggable! {
    DatabaseError {
        #[error("Failed to create database")]
        CreateDatabaseFailed => tracing::Level::ERROR,

        #[error("Failed to connect to database")]
        DatabaseConnectFailed => tracing::Level::ERROR,

        #[error("Failed to lock database")]
        LockDatabaseFailed => tracing::Level::ERROR,

        #[error("Failed to unlock database")]
        UnlockDatabaseFailed => tracing::Level::ERROR,
    }
}
