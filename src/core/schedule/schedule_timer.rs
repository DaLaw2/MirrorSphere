use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::communication_manager::CommunicationManager;
use crate::interface::communication::command::CommandHandler;
use crate::interface::core::runnable::Runnable;
use crate::model::core::schedule::communication::*;
use crate::model::core::schedule::schedule::ScheduleState;
use crate::model::error::Error;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tokio::select;
use tokio::sync::oneshot::Receiver;
use tokio::sync::Notify;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;
use tracing::error;

pub struct ScheduleTimer {
    app_config: Arc<AppConfig>,
    communication_manager: Arc<CommunicationManager>,
    refresh_notify: Arc<Notify>,
}

impl ScheduleTimer {
    pub fn new(
        app_config: Arc<AppConfig>,
        communication_manager: Arc<CommunicationManager>,
    ) -> Self {
        ScheduleTimer {
            app_config,
            communication_manager,
            refresh_notify: Arc::new(Notify::new()),
        }
    }

    pub async fn register_services(self: Arc<Self>) {
        let communication_manager = self.communication_manager.clone();
        communication_manager
            .with_service(self)
            .command::<ScheduleTimerCommand>()
            .build();
    }

    async fn calculate_sleep_duration(&self) -> Result<Option<Duration>, Error> {
        let mut next_time = None;
        let communication_manager = self.communication_manager.clone();
        let response = communication_manager
            .send_query(ScheduleManagerQuery::GetSchedules)
            .await?;
        let ScheduleManagerQueryResponse::GetSchedules(schedules) = response;
        for schedule in schedules {
            if schedule.state != ScheduleState::Active {
                continue;
            }
            if let Some(schedule_next_time) = schedule.next_run_time {
                match next_time {
                    Some(current_time) => {
                        if schedule_next_time < current_time {
                            next_time = Some(schedule_next_time);
                        }
                    }
                    None => next_time = Some(schedule_next_time),
                }
            }
        }
        if let Some(schedule_next_time) = next_time {
            let now = Utc::now().naive_utc();
            let duration = schedule_next_time.signed_duration_since(now);
            Ok(Some(Duration::seconds(duration.num_seconds().max(0))))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl Runnable for ScheduleTimer {
    async fn run_impl(self: Arc<Self>, mut shutdown_rx: Receiver<()>) {
        let communication_manager = self.communication_manager.clone();

        loop {
            let mut sleep_time = match self.calculate_sleep_duration().await {
                Ok(Some(duration)) => duration,
                Ok(None) => Duration::seconds(self.app_config.default_wakeup_time),
                Err(err) => {
                    error!("{}", err);
                    Duration::seconds(self.app_config.default_wakeup_time)
                }
            };
            if sleep_time < Duration::seconds(0) {
                sleep_time = Duration::seconds(0);
            }
            select! {
                biased;
                _ = &mut shutdown_rx => { break; }
                _ = self.refresh_notify.notified() => { continue; }
                _ = sleep(sleep_time.to_std().unwrap()) => {}
            }
            if let Err(err) = communication_manager
                .send_command(ScheduleManagerCommand::ExecuteReadySchedules)
                .await
            {
                error!("{}", err);
            }
        }
    }
}

#[async_trait]
impl CommandHandler<ScheduleTimerCommand> for ScheduleTimer {
    async fn handle_command(&self, command: ScheduleTimerCommand) -> Result<(), Error> {
        match command {
            ScheduleTimerCommand::RefreshTimer => {
                self.refresh_notify.notify_one();
                Ok(())
            }
        }
    }
}
