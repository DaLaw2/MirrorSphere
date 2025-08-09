use crate::core::infrastructure::actor_system::ActorSystem;
use crate::core::infrastructure::app_config::AppConfig;
use crate::core::schedule::schedule_service::ScheduleService;
use crate::model::core::schedule::backup_schedule::ScheduleState;
use crate::model::core::schedule::message::*;
use crate::model::error::actor::ActorError;
use crate::model::error::Error;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;
use tracing::error;

pub struct ScheduleTimer {
    app_config: Arc<AppConfig>,
    actor_system: Arc<ActorSystem>,
}

impl ScheduleTimer {
    pub fn new(app_config: Arc<AppConfig>, actor_system: Arc<ActorSystem>) -> Self {
        ScheduleTimer {
            app_config,
            actor_system,
        }
    }

    pub async fn run(
        self: Arc<Self>,
    ) -> Result<(mpsc::UnboundedSender<()>, oneshot::Sender<()>), Error> {
        let (refresh_tx, mut refresh_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let service_ref = self
            .actor_system
            .actor_of::<ScheduleService>()
            .ok_or(ActorError::ActorNotFound)?;
        tokio::spawn(async move {
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
                    _ = refresh_rx.recv() => { continue; }
                    _ = sleep(sleep_time.to_std().unwrap()) => {}
                }
                if let Err(err) = service_ref
                    .tell(ScheduleServiceMessage::UnitNotification(
                        UnitNotificationMessage::CheckSchedule,
                    ))
                    .await
                {
                    error!("{}", err);
                }
            }
        });
        Ok((refresh_tx, shutdown_tx))
    }

    async fn calculate_sleep_duration(&self) -> Result<Option<Duration>, Error> {
        let mut next_time = None;
        let service_ref = self
            .actor_system
            .actor_of::<ScheduleService>()
            .ok_or(ActorError::ActorNotFound)?;
        let response = service_ref
            .ask(ScheduleServiceMessage::ServiceCall(
                ServiceCallMessage::GetSchedules,
            ))
            .await?;
        let ScheduleServiceResponse::ServiceCall(service_call) = response else {
            return Ok(None);
        };
        let ServiceCallResponse::GetSchedules(schedules) = service_call;
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
