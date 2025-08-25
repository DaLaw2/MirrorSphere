use crate::core::infrastructure::communication_manager::CommunicationManager;
use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::interface::communication::command::CommandHandler;
use crate::interface::communication::query::QueryHandler;
use crate::interface::repository::schedule::ScheduleRepository;
use crate::model::core::backup::communication::BackupCommand;
use crate::model::core::schedule::schedule::*;
use crate::model::core::schedule::communication::*;
use crate::model::error::Error;
use async_trait::async_trait;
use chrono::{Duration, Months, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct ScheduleManager {
    database_manager: Arc<DatabaseManager>,
    communication_manager: Arc<CommunicationManager>,
    schedules: DashMap<Uuid, Schedule>,
}

impl ScheduleManager {
    pub async fn new(
        database_manager: Arc<DatabaseManager>,
        communication_manager: Arc<CommunicationManager>,
    ) -> Result<Self, Error> {
        let schedules = DashMap::new();
        let database_schedules = database_manager.get_all_backup_schedules().await?;
        for schedule in database_schedules {
            schedules.insert(schedule.uuid, schedule);
        }
        let schedule_manager = ScheduleManager {
            database_manager,
            communication_manager,
            schedules,
        };
        Ok(schedule_manager)
    }

    pub async fn register_services(self: Arc<Self>) {
        let communication_manager = self.communication_manager.clone();
        communication_manager
            .with_service(self)
            .command::<ScheduleManagerCommand>()
            .query::<ScheduleManagerQuery>()
            .build();
    }

    pub async fn get_all_schedules(&self) -> Vec<Schedule> {
        self.schedules.iter().map(|x| x.value().clone()).collect()
    }

    pub async fn create_schedule(&self, schedule: Schedule) -> Result<(), Error> {
        self.database_manager
            .create_backup_schedule(&schedule)
            .await?;
        self.schedules.insert(schedule.uuid, schedule);
        Ok(())
    }

    pub async fn modify_schedule(&self, schedule: Schedule) -> Result<(), Error> {
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
                let command = BackupCommand::AddExecution(execution);
                self.communication_manager.send_command(command).await?;
                self.update_next_run_time(schedule);
                database_manager.modify_backup_schedule(schedule).await?;
            }
        }

        Ok(())
    }

    fn update_next_run_time(&self, schedule: &mut Schedule) {
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
impl CommandHandler<ScheduleManagerCommand> for ScheduleManager {
    async fn handle_command(&self, command: ScheduleManagerCommand) -> Result<(), Error> {
        match command {
            ScheduleManagerCommand::AddSchedule(schedule) => {
                self.create_schedule(schedule).await?;
            }
            ScheduleManagerCommand::ModifySchedule(schedule) => {
                self.modify_schedule(schedule).await?;
            }
            ScheduleManagerCommand::RemoveSchedule(uuid) => {
                self.remove_schedule(uuid).await?;
            }
            ScheduleManagerCommand::ActivateSchedule(uuid) => {
                self.active_schedule(uuid).await?;
            }
            ScheduleManagerCommand::PauseSchedule(uuid) => {
                self.pause_schedule(uuid).await?;
            }
            ScheduleManagerCommand::DisableSchedule(uuid) => {
                self.disable_schedule(uuid).await?;
            }
            ScheduleManagerCommand::ExecuteReadySchedules => {
                self.execute_ready_schedule().await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl QueryHandler<ScheduleManagerQuery> for ScheduleManager {
    async fn handle_query(
        &self,
        query: ScheduleManagerQuery,
    ) -> Result<ScheduleManagerQueryResponse, Error> {
        match query {
            ScheduleManagerQuery::GetSchedules => {
                let executions = self.get_all_schedules().await;
                Ok(ScheduleManagerQueryResponse::GetSchedules(executions))
            }
        }
    }
}
