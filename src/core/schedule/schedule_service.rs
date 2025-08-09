use crate::core::infrastructure::actor_system::ActorSystem;
use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::core::schedule::schedule_manager::ScheduleManager;
use crate::core::schedule::schedule_timer::ScheduleTimer;
use crate::interface::actor::actor::Actor;
use crate::interface::actor::message::Message;
use crate::model::core::schedule::message::*;
use crate::model::error::system::SystemError;
use crate::model::error::Error;
use async_trait::async_trait;
use macros::log;
use std::sync::{Arc, OnceLock};
use tokio::sync::{mpsc, oneshot};

pub struct ScheduleService {
    schedule_manager: Arc<ScheduleManager>,
    schedule_timer: Arc<ScheduleTimer>,
    timer_refresh: OnceLock<mpsc::UnboundedSender<()>>,
    shutdowns: Vec<oneshot::Sender<()>>,
}

impl ScheduleService {
    pub async fn init(
        app_config: Arc<AppConfig>,
        database_manager: Arc<DatabaseManager>,
        actor_system: Arc<ActorSystem>,
    ) -> Result<(), Error> {
        let schedule_manager =
            Arc::new(ScheduleManager::new(database_manager, actor_system.clone()).await?);
        let schedule_timer = Arc::new(ScheduleTimer::new(app_config, actor_system.clone()));
        let schedule_service = Self {
            schedule_manager,
            schedule_timer,
            timer_refresh: OnceLock::new(),
            shutdowns: Vec::new(),
        };
        actor_system.spawn(schedule_service).await;
        Ok(())
    }

    pub fn refresh_timer(&self) {
        if let Some(timer_refresh) = self.timer_refresh.get() {
            let _ = timer_refresh.send(());
        }
    }
}

#[async_trait]
impl Actor for ScheduleService {
    type Message = ScheduleServiceMessage;

    async fn pre_start(&mut self) {
        let schedule_timer = self.schedule_timer.clone();
        if let Ok((timer_refresh, shutdown)) = schedule_timer.run().await {
            self.timer_refresh.get_or_init(|| timer_refresh);
            self.shutdowns.push(shutdown);
        }
    }

    async fn post_stop(&mut self) {
        let shutdowns = std::mem::take(&mut self.shutdowns);
        for shutdown in shutdowns {
            if let Err(_) = shutdown.send(()) {
                log!(SystemError::ShutdownSignalFailed)
            }
        }
    }

    async fn receive(
        &mut self,
        message: Self::Message,
    ) -> Result<<Self::Message as Message>::Response, Error> {
        match message {
            ScheduleServiceMessage::UnitNotification(unit_notification) => {
                match unit_notification {
                    UnitNotificationMessage::RefreshTimer => {
                        self.refresh_timer();
                        Ok(ScheduleServiceResponse::None)
                    }
                    UnitNotificationMessage::CheckSchedule => {
                        self.schedule_manager.execute_ready_schedule().await?;
                        Ok(ScheduleServiceResponse::None)
                    }
                }
            }
            ScheduleServiceMessage::ServiceCall(service_call) => match service_call {
                ServiceCallMessage::AddSchedule(schedule) => {
                    self.schedule_manager.create_schedule(schedule).await?;
                    self.refresh_timer();
                    Ok(ScheduleServiceResponse::None)
                }
                ServiceCallMessage::ModifySchedule(schedule) => {
                    self.schedule_manager.modify_schedule(schedule).await?;
                    self.refresh_timer();
                    Ok(ScheduleServiceResponse::None)
                }
                ServiceCallMessage::RemoveSchedule(uuid) => {
                    self.schedule_manager.remove_schedule(uuid).await?;
                    self.refresh_timer();
                    Ok(ScheduleServiceResponse::None)
                }
                ServiceCallMessage::ActivateSchedule(uuid) => {
                    self.schedule_manager.active_schedule(uuid).await?;
                    self.refresh_timer();
                    Ok(ScheduleServiceResponse::None)
                }
                ServiceCallMessage::PauseSchedule(uuid) => {
                    self.schedule_manager.pause_schedule(uuid).await?;
                    self.refresh_timer();
                    Ok(ScheduleServiceResponse::None)
                }
                ServiceCallMessage::DisableSchedule(uuid) => {
                    self.schedule_manager.disable_schedule(uuid).await?;
                    self.refresh_timer();
                    Ok(ScheduleServiceResponse::None)
                }
                ServiceCallMessage::GetSchedules => {
                    let schedules = self.schedule_manager.get_all_schedules().await;
                    Ok(ScheduleServiceResponse::ServiceCall(
                        ServiceCallResponse::GetSchedules(schedules),
                    ))
                }
            },
        }
    }
}
