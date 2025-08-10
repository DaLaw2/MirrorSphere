use uuid::Uuid;
use crate::interface::actor::message::Message;
use crate::model::core::schedule::backup_schedule::BackupSchedule;

pub enum ScheduleServiceMessage {
    UnitNotification(UnitNotificationMessage),
    ServiceCall(ServiceCallMessage),
}

pub enum ScheduleServiceResponse {
    ServiceCall(ServiceCallResponse),
    None,
}

pub enum UnitNotificationMessage {
    CheckSchedule,
}

pub enum ServiceCallMessage {
    AddSchedule(BackupSchedule),
    ModifySchedule(BackupSchedule),
    RemoveSchedule(Uuid),
    ActivateSchedule(Uuid),
    PauseSchedule(Uuid),
    DisableSchedule(Uuid),
    GetSchedules,
}

pub enum ServiceCallResponse {
    GetSchedules(Vec<BackupSchedule>),
}

impl Message for ScheduleServiceMessage {
    type Response = ScheduleServiceResponse;
}
