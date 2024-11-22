use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseEntry {
    #[error("Connect to database successfully")]
    DatabaseConnectSuccess,
    #[error("Failed to connect database")]
    DatabaseConnectFailed,
}
