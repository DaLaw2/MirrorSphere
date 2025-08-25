use uuid::Uuid;
use crate::interface::communication::command::Command;
use crate::interface::communication::message::Message;
use crate::interface::communication::query::Query;
use crate::model::core::schedule::schedule::Schedule;

pub enum ScheduleManagerCommand {
    AddSchedule(Schedule),
    ModifySchedule(Schedule),
    RemoveSchedule(Uuid),
    ActivateSchedule(Uuid),
    PauseSchedule(Uuid),
    DisableSchedule(Uuid),
    ExecuteReadySchedules,
}

impl Message for ScheduleManagerCommand {
    type Response = ();
}

impl Command for ScheduleManagerCommand {}

pub enum ScheduleManagerQuery {
    GetSchedules,
}

impl Message for ScheduleManagerQuery {
    type Response = ScheduleManagerQueryResponse;
}

impl Query for ScheduleManagerQuery {}

pub enum ScheduleManagerQueryResponse {
    GetSchedules(Vec<Schedule>),
}

pub enum ScheduleTimerCommand {
    RefreshTimer
}

impl Message for ScheduleTimerCommand {
    type Response = ();
}

impl Command for ScheduleTimerCommand {}
