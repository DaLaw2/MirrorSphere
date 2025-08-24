use crate::interface::communication::command::Command;
use crate::interface::communication::message::Message;
use crate::interface::communication::query::Query;
use crate::model::core::backup::backup_execution::BackupExecution;
use uuid::Uuid;

pub enum BackupCommand {
    AddExecution(BackupExecution),
    RemoveExecution(Uuid),
    StartExecution(Uuid),
    SuspendExecution(Uuid),
    ResumeExecution(Uuid),
}

impl Message for BackupCommand {
    type Response = ();
}

impl Command for BackupCommand {}

pub enum BackupInternalCommand {}

impl Message for BackupInternalCommand {
    type Response = ();
}

impl Command for BackupInternalCommand {}

pub enum BackupQuery {
    GetExecutions,
}

impl Message for BackupQuery {
    type Response = BackupQueryResponse;
}

impl Query for BackupQuery {}

pub enum BackupQueryResponse {
    GetExecutions(Vec<(Uuid, BackupExecution)>),
}
