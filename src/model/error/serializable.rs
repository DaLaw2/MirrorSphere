use crate::model::error::database::DatabaseError;
use crate::model::error::event::EventError;
use crate::model::error::io::IOError;
use crate::model::error::misc::MiscError;
use crate::model::error::system::SystemError;
use crate::model::error::task::TaskError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableError {
    Database(DatabaseError),
    Event(EventError),
    IO(IOError),
    Misc(MiscError),
    System(SystemError),
    Task(TaskError),
}

impl From<DatabaseError> for SerializableError {
    fn from(error: DatabaseError) -> Self {
        Self::Database(error)
    }
}

impl From<EventError> for SerializableError {
    fn from(error: EventError) -> Self {
        Self::Event(error)
    }
}

impl From<IOError> for SerializableError {
    fn from(error: IOError) -> Self {
        Self::IO(error)
    }
}

impl From<MiscError> for SerializableError {
    fn from(error: MiscError) -> Self {
        Self::Misc(error)
    }
}

impl From<SystemError> for SerializableError {
    fn from(error: SystemError) -> Self {
        Self::System(error)
    }
}

impl From<TaskError> for SerializableError {
    fn from(error: TaskError) -> Self {
        Self::Task(error)
    }
}
