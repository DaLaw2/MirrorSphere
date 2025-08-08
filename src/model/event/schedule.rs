use uuid::Uuid;
use crate::interface::event::Event;
use crate::model::core::schedule::backup_schedule::BackupSchedule;

#[derive(Clone, Debug)]
pub struct ScheduleCreateRequest {
    pub schedule: BackupSchedule,
}
impl Event for ScheduleCreateRequest {}

#[derive(Clone, Debug)]
pub struct ScheduleModifyRequest {
    pub schedule: BackupSchedule,
}
impl Event for ScheduleModifyRequest {}

#[derive(Clone, Debug)]
pub struct ScheduleRemoveRequest {
    pub schedule_id: Uuid,
}
impl Event for ScheduleRemoveRequest {}

#[derive(Clone, Debug)]
pub struct ScheduleActiveRequest {
    pub schedule_id: Uuid,
}
impl Event for ScheduleActiveRequest {}

#[derive(Clone, Debug)]
pub struct SchedulePauseRequest {
    pub schedule_id: Uuid,
}
impl Event for SchedulePauseRequest {}

#[derive(Clone, Debug)]
pub struct ScheduleDisableRequest {
    pub schedule_id: Uuid,
}
impl Event for ScheduleDisableRequest {}
