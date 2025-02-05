use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseEntry {
    #[error("Failed to create database")]
    CreateDatabaseFailed,
    #[error("Connect to database successfully")]
    DatabaseConnectSuccess,
    #[error("Failed to connect database")]
    DatabaseConnectFailed,
    #[error("Lock database failed")]
    LockDatabaseFailed,
    #[error("Unlock database failed")]
    UnlockDatabaseFailed,
}
