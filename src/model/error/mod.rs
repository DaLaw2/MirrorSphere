pub mod database;
pub mod io;
pub mod misc;
pub mod system;
pub mod task;

use crate::model::error::database::DatabaseError;
use crate::model::error::io::IOError;
use crate::model::error::misc::MiscError;
use crate::model::error::system::SystemError;
use crate::model::error::task::TaskError;
use serde::{Deserialize, Serialize};
use tracing::Level;

#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize)]
pub enum Error {
    #[error("{0}")]
    Database(DatabaseError),
    #[error("{0}")]
    IO(IOError),
    #[error("{0}")]
    Misc(MiscError),
    #[error("{0}")]
    System(SystemError),
    #[error("{0}")]
    Task(TaskError),
}

impl Error {
    pub fn level(&self) -> Level {
        match self {
            Error::Database(err) => err.level(),
            Error::IO(err) => err.level(),
            Error::Misc(err) => err.level(),
            Error::System(err) => err.level(),
            Error::Task(err) => err.level(),
        }
    }
}

impl From<DatabaseError> for Error {
    fn from(error: DatabaseError) -> Self {
        Self::Database(error)
    }
}

impl From<IOError> for Error {
    fn from(error: IOError) -> Self {
        Self::IO(error)
    }
}

impl From<MiscError> for Error {
    fn from(error: MiscError) -> Self {
        Self::Misc(error)
    }
}

impl From<SystemError> for Error {
    fn from(error: SystemError) -> Self {
        Self::System(error)
    }
}

impl From<TaskError> for Error {
    fn from(error: TaskError) -> Self {
        Self::Task(error)
    }
}
