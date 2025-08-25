use crate::interface::communication::command::Command;
use crate::interface::communication::event::Event;
use crate::interface::communication::message::Message;
use crate::interface::communication::query::Query;
use crate::model::core::backup::execution::Execution;
use crate::model::error::Error;
use uuid::Uuid;

pub enum BackupCommand {
    AddExecution(Execution),
    RemoveExecution(Uuid),
    StartExecution(Uuid),
    SuspendExecution(Uuid),
    ResumeExecution(Uuid),
}

impl Message for BackupCommand {
    type Response = ();
}

impl Command for BackupCommand {}

pub enum BackupQuery {
    GetExecutions,
}

impl Message for BackupQuery {
    type Response = BackupQueryResponse;
}

impl Query for BackupQuery {}

pub enum BackupQueryResponse {
    GetExecutions(Vec<(Uuid, Execution)>),
}

#[derive(Clone)]
pub struct ExecutionErrorEvent {
    pub uuid: Uuid,
    pub errors: Vec<Error>,
}

impl Event for ExecutionErrorEvent {}
