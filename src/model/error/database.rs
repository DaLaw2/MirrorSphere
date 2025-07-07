use crate::traceable;

traceable! {
    DatabaseError {
        #[error("Failed to create database")]
        CreateDatabaseFailed => tracing::Level::ERROR,

        #[error("Failed to connect to database")]
        DatabaseConnectFailed => tracing::Level::ERROR,

        #[error("Failed to lock database")]
        LockDatabaseFailed => tracing::Level::ERROR,

        #[error("Failed to unlock database")]
        UnlockDatabaseFailed => tracing::Level::ERROR,

        #[error("Failed to execute SQL statement")]
        StatementExecutionFailed  => tracing::Level::ERROR,
    }
}
