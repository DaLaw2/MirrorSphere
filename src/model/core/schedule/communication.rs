use uuid::Uuid;
use crate::interface::communication::command::Command;
use crate::interface::communication::message::Message;
use crate::interface::communication::query::Query;
use crate::model::core::schedule::backup_schedule::BackupSchedule;

pub enum ScheduleCommand {
    AddSchedule(BackupSchedule),
    ModifySchedule(BackupSchedule),
    RemoveSchedule(Uuid),
    ActivateSchedule(Uuid),
    PauseSchedule(Uuid),
    DisableSchedule(Uuid),
}

impl Message for ScheduleCommand {
    type Response = ();
}

impl Command for ScheduleCommand {}

pub enum ScheduleInternalCommand {
    TimerNotify,
    RefreshTimer,
}

impl Message for ScheduleInternalCommand {
    type Response = ();
}

impl Command for ScheduleInternalCommand {}

pub enum ScheduleQuery {
    GetSchedules,
}

impl Message for ScheduleQuery {
    type Response = ScheduleQueryResponse;
}

impl Query for ScheduleQuery {}

pub enum ScheduleQueryResponse {
    GetSchedules(Vec<BackupSchedule>),
}
