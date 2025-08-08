use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use chrono::{Duration, Utc};
use macros::log;
use tracing::error;
use tokio::select;
use tokio::time::sleep;
use crate::core::infrastructure::app_config::AppConfig;
use crate::core::schedule::schedule_manager::ScheduleManager;
use crate::model::core::schedule::backup_schedule::ScheduleState;
use crate::model::error::Error;
use crate::model::error::task::TaskError;

pub struct ScheduleTimer {
    app_config: Arc<AppConfig>,
    schedule_manager: Arc<ScheduleManager>,
    shutdown_rx: Option<oneshot::Receiver<()>>,
    refresh_rx: mpsc::UnboundedReceiver<()>,
}

impl ScheduleTimer {
    pub fn new(
        app_config: Arc<AppConfig>,
        schedule_manager: Arc<ScheduleManager>,
        shutdown_rx: oneshot::Receiver<()>,
        refresh_rx: mpsc::UnboundedReceiver<()>,
    ) -> Self {
        ScheduleTimer {
            app_config,
            schedule_manager,
            shutdown_rx: Some(shutdown_rx),
            refresh_rx,
        }
    }

    pub async fn run(mut self) {
        let schedule_manager = self.schedule_manager.clone();
        match self.shutdown_rx.take() {
            Some(mut shutdown_rx) => loop {
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
                    _ = self.refresh_rx.recv() => {}
                    _ = sleep(sleep_time.to_std().unwrap()) => {}
                }
                if let Err(err) = schedule_manager.execute_ready_schedule().await {
                    error!("{}", err);
                }
            },
            None => log!(TaskError::IllegalRunState),
        }
    }

    async fn calculate_sleep_duration(&self) -> Result<Option<Duration>, Error> {
        let mut next_time = None;

        let schedules = self.schedule_manager.get_all_schedules().await?;

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