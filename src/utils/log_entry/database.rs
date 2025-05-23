use crate::utils::log_entry::define_log_entries;

define_log_entries! {
    DatabaseEntry {
        #[error("Failed to create database")]
        CreateDatabaseFailed: tracing::Level::ERROR,

        #[error("Connected to database successfully")]
        DatabaseConnectSuccess: tracing::Level::INFO,

        #[error("Failed to connect to database")]
        DatabaseConnectFailed: tracing::Level::ERROR,

        #[error("Failed to lock database")]
        LockDatabaseFailed: tracing::Level::ERROR,

        #[error("Failed to unlock database")]
        UnlockDatabaseFailed: tracing::Level::ERROR,
    }
}
