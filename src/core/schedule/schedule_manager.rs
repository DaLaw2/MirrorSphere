use crate::core::backup::backup_service::BackupService;
use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::interface::repository::schedule::ScheduleRepository;
use crate::model::core::schedule::backup_schedule::*;
use crate::model::error::Error;
use chrono::{Duration, Months, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;
use crate::interface::communication::message::Message;
use crate::interface::core::unit::Unit;
use crate::model::core::schedule::communication::{ScheduleCommand, ScheduleInternalCommand, ScheduleQuery, ScheduleQueryResponse};

pub struct ScheduleManager {
    database_manager: Arc<DatabaseManager>,
    schedules: DashMap<Uuid, BackupSchedule>,
}

impl ScheduleManager {
    pub async fn new(
        database_manager: Arc<DatabaseManager>,
    ) -> Result<Self, Error> {
        let schedules = DashMap::new();
        let database_schedules = database_manager.get_all_backup_schedules().await?;
        for schedule in database_schedules {
            schedules.insert(schedule.uuid, schedule);
        }
        let schedule_manager = ScheduleManager {
            database_manager,
            actor_system,
            schedules,
        };
        Ok(schedule_manager)
    }

    pub async fn get_all_schedules(&self) -> Vec<BackupSchedule> {
        self.schedules.iter().map(|x| x.value().clone()).collect()
    }

    pub async fn create_schedule(&self, schedule: BackupSchedule) -> Result<(), Error> {
        self.database_manager
            .create_backup_schedule(&schedule)
            .await?;
        self.schedules.insert(schedule.uuid, schedule);
        Ok(())
    }

    pub async fn modify_schedule(&self, schedule: BackupSchedule) -> Result<(), Error> {
        self.database_manager
            .modify_backup_schedule(&schedule)
            .await?;
        self.schedules.insert(schedule.uuid, schedule);
        Ok(())
    }

    pub async fn remove_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        self.database_manager.remove_backup_schedule(uuid).await?;
        self.schedules.remove(&uuid);
        Ok(())
    }

    pub async fn active_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        if let Some(mut schedule) = self.database_manager.get_backup_schedule(uuid).await? {
            schedule.state = ScheduleState::Active;
            self.database_manager
                .modify_backup_schedule(&schedule)
                .await?;
            self.schedules.insert(schedule.uuid, schedule);
        }
        Ok(())
    }

    pub async fn pause_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        if let Some(mut schedule) = self.database_manager.get_backup_schedule(uuid).await? {
            schedule.state = ScheduleState::Paused;
            self.database_manager
                .modify_backup_schedule(&schedule)
                .await?;
            self.schedules.insert(schedule.uuid, schedule);
        }
        Ok(())
    }

    pub async fn disable_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        if let Some(mut schedule) = self.database_manager.get_backup_schedule(uuid).await? {
            schedule.state = ScheduleState::Disabled;
            self.database_manager
                .modify_backup_schedule(&schedule)
                .await?;
            self.schedules.insert(schedule.uuid, schedule);
        }
        Ok(())
    }

    pub async fn execute_ready_schedule(&self) -> Result<(), Error> {
        let database_manager = self.database_manager.clone();

        let now = Utc::now().naive_utc();
        let mut schedules = self.get_all_schedules().await;

        for schedule in schedules.iter_mut() {
            if schedule.state != ScheduleState::Active {
                continue;
            }
            if let Some(next_run_time) = schedule.next_run_time {
                if next_run_time >= now {
                    continue;
                }
                let execution = schedule.to_execution();
                if let Some(service_ref) = self.actor_system.actor_of::<BackupService>() {
                    service_ref
                        .tell(BackupServiceMessage::ServiceCall(
                            ServiceCallMessage::AddExecution(execution),
                        ))
                        .await?;
                }
                self.update_next_run_time(schedule);
                database_manager.modify_backup_schedule(schedule).await?;
            }
        }

        Ok(())
    }

    fn update_next_run_time(&self, schedule: &mut BackupSchedule) {
        if schedule.next_run_time.is_none() {
            return;
        }
        let now = Utc::now().naive_utc();
        let old_next_run_time = schedule.next_run_time.unwrap();
        let new_next_run_time = match schedule.interval {
            ScheduleInterval::Once => None,
            ScheduleInterval::Daily => Some(old_next_run_time + Duration::days(1)),
            ScheduleInterval::Weekly => Some(old_next_run_time + Duration::days(7)),
            ScheduleInterval::Monthly => Some(
                old_next_run_time
                    .checked_add_months(Months::new(1))
                    .unwrap_or(old_next_run_time + Duration::days(30)),
            ),
        };
        schedule.last_run_time = Some(now);
        schedule.next_run_time = new_next_run_time;
    }
}

#[async_trait]
impl Unit for ScheduleManager {
    type Command = ScheduleCommand;
    type InternalCommand = ScheduleInternalCommand;
    type Query = ScheduleQuery;

    fn get_internal_channel(&self) -> UnboundedReceiver<Self::InternalCommand> {
        todo!()
    }

    async fn handle_command(&self, command: Self::Command) -> Result<(), Error> {
        match command {
            ScheduleCommand::AddSchedule(schedule) => {
                self.create_schedule(schedule).await?;
            }
            ScheduleCommand::ModifySchedule(schedule) => {
                self.modify_schedule(schedule).await?;
            }
            ScheduleCommand::RemoveSchedule(uuid) => {
                self.remove_schedule(uuid).await?;
            }
            ScheduleCommand::ActivateSchedule(uuid) => {
                self.active_schedule(uuid).await?;
            }
            ScheduleCommand::PauseSchedule(uuid) => {
                self.pause_schedule(uuid).await?;
            }
            ScheduleCommand::DisableSchedule(uuid) => {
                self.disable_schedule(uuid).await?;
            }
        }
        Ok(())
    }

    async fn handle_internal_command(&self, command: Self::InternalCommand) -> Result<(), Error> {
        match command {
            ScheduleInternalCommand::TimerNotify => {
                self.execute_ready_schedule().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_query(&self, query: Self::Query) -> Result<<Self::Query as Message>::Response, Error> {
        match query {
            ScheduleQuery::GetSchedules => {
                let executions = self.get_all_schedules().await;
                Ok(ScheduleQueryResponse::GetSchedules(executions))
            }
        }
    }
}
