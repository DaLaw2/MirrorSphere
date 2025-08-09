use crate::interface::actor::message::Message;
use crate::model::core::backup::backup_execution::BackupExecution;
use uuid::Uuid;

pub enum BackupServiceMessage {
    ServiceCall(ServiceCallMessage),
}

pub enum BackupServiceResponse {
    ServiceCall(ServiceCallResponse),
    None,
}

impl Message for BackupServiceMessage {
    type Response = BackupServiceResponse;
}

pub enum ServiceCallMessage {
    AddExecution(BackupExecution),
    RemoveExecution(Uuid),
    StartExecution(Uuid),
    SuspendExecution(Uuid),
    ResumeExecution(Uuid),
    GetExecutions,
}

pub enum ServiceCallResponse {
    GetExecutions(Vec<(Uuid, BackupExecution)>),
    None,
}
