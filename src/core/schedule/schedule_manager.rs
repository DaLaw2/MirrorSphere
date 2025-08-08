use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::core::event_bus::EventBus;
use crate::interface::repository::schedule::ScheduleRepository;
use crate::interface::service_unit::ServiceUnit;
use crate::model::backup::backup_schedule::*;
use crate::model::error::Error;
use crate::model::error::system::SystemError;
use crate::model::error::task::TaskError;
use crate::model::event::execution::*;
use crate::model::event::schedule::*;
use crate::model::log::system::SystemLog;
use async_trait::async_trait;
use chrono::{Duration, Months, Utc};
use macros::log;
use std::sync::Arc;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;
use tracing::error;
use uuid::Uuid;

pub struct ScheduleManager {
    app_config: Arc<AppConfig>,
    event_bus: Arc<EventBus>,
    database_manager: Arc<DatabaseManager>,
}

impl ScheduleManager {
    pub fn new(
        app_config: Arc<AppConfig>,
        event_bus: Arc<EventBus>,
        database_manager: Arc<DatabaseManager>,
    ) -> Self {
        ScheduleManager {
            app_config,
            event_bus,
            database_manager,
        }
    }

    pub async fn get_all_schedules(&self) -> Result<Vec<BackupSchedule>, Error> {
        self.database_manager.get_all_backup_schedules().await
    }

    pub async fn create_schedule(&self, schedule: BackupSchedule) -> Result<(), Error> {
        self.database_manager
            .create_backup_schedule(&schedule)
            .await
    }

    pub async fn modify_schedule(&self, schedule: BackupSchedule) -> Result<(), Error> {
        self.database_manager
            .modify_backup_schedule(&schedule)
            .await
    }

    pub async fn remove_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        self.database_manager.remove_backup_schedule(uuid).await
    }

    pub async fn active_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        if let Some(mut schedule) = self.database_manager.get_backup_schedule(uuid).await? {
            schedule.state = ScheduleState::Active;
            self.database_manager
                .modify_backup_schedule(&schedule)
                .await?;
        }
        Ok(())
    }

    pub async fn pause_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        if let Some(mut schedule) = self.database_manager.get_backup_schedule(uuid).await? {
            schedule.state = ScheduleState::Paused;
            self.database_manager
                .modify_backup_schedule(&schedule)
                .await?;
        }
        Ok(())
    }

    pub async fn disable_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        if let Some(mut schedule) = self.database_manager.get_backup_schedule(uuid).await? {
            schedule.state = ScheduleState::Disabled;
            self.database_manager
                .modify_backup_schedule(&schedule)
                .await?;
        }
        Ok(())
    }

    pub async fn execute_ready_schedule(&self) -> Result<(), Error> {
        let event_bus = self.event_bus.clone();
        let database_manager = self.database_manager.clone();

        let now = Utc::now().naive_utc();
        let mut schedules = self.get_all_schedules().await?;

        for schedule in schedules.iter_mut() {
            if schedule.state != ScheduleState::Active {
                continue;
            }
            if let Some(next_run_time) = schedule.next_run_time {
                if next_run_time < now {
                    let execution = schedule.to_execution();
                    event_bus.publish(ExecutionAddRequest { execution });
                    self.update_next_run_time(schedule);
                    database_manager.modify_backup_schedule(schedule).await?;
                }
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

struct ScheduleTimer {
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

#[async_trait]
impl ServiceUnit for ScheduleManager {
    async fn run_impl(self: Arc<Self>, mut shutdown_rx: oneshot::Receiver<()>) {
        let (timer_shutdown_tx, timer_shutdown_rx) = oneshot::channel();
        let (timer_refresh_tx, timer_refresh_rx) = mpsc::unbounded_channel();

        let schedule_timer = ScheduleTimer::new(
            self.app_config.clone(),
            self.clone(),
            timer_shutdown_rx,
            timer_refresh_rx,
        );

        tokio::spawn(schedule_timer.run());

        let event_bus = self.event_bus.clone();
        let schedule_manager = self.clone();
        let create_schedule = event_bus.subscribe::<ScheduleCreateRequest>();
        let modify_schedule = event_bus.subscribe::<ScheduleModifyRequest>();
        let remove_schedule = event_bus.subscribe::<ScheduleRemoveRequest>();
        let active_schedule = event_bus.subscribe::<ScheduleActiveRequest>();
        let pause_schedule = event_bus.subscribe::<SchedulePauseRequest>();
        let disable_schedule = event_bus.subscribe::<ScheduleDisableRequest>();
        let sleep_duration = Duration::milliseconds(self.app_config.internal_timestamp)
            .to_std()
            .unwrap();

        loop {
            if shutdown_rx.try_recv().is_ok() {
                log!(SystemLog::Terminating);
                if timer_shutdown_tx.send(()).is_err() {
                    log!(SystemError::TerminateError(
                        "Fail send shutdown signal to timer"
                    ))
                } else {
                    log!(SystemLog::TerminateComplete);
                }
                break;
            }

            let mut need_refresh = false;

            while let Ok(event) = create_schedule.try_recv() {
                match schedule_manager.create_schedule(event.schedule).await {
                    Ok(_) => need_refresh = true,
                    Err(err) => error!("{}", err),
                }
            }
            while let Ok(event) = modify_schedule.try_recv() {
                match schedule_manager.modify_schedule(event.schedule).await {
                    Ok(_) => need_refresh = true,
                    Err(err) => error!("{}", err),
                }
            }
            while let Ok(event) = remove_schedule.try_recv() {
                match schedule_manager.remove_schedule(event.schedule_id).await {
                    Ok(_) => need_refresh = true,
                    Err(err) => error!("{}", err),
                }
            }
            while let Ok(event) = active_schedule.try_recv() {
                match schedule_manager.active_schedule(event.schedule_id).await {
                    Ok(_) => need_refresh = true,
                    Err(err) => error!("{}", err),
                }
            }
            while let Ok(event) = pause_schedule.try_recv() {
                match schedule_manager.pause_schedule(event.schedule_id).await {
                    Ok(_) => need_refresh = true,
                    Err(err) => error!("{}", err),
                }
            }
            while let Ok(event) = disable_schedule.try_recv() {
                match schedule_manager.disable_schedule(event.schedule_id).await {
                    Ok(_) => need_refresh = true,
                    Err(err) => error!("{}", err),
                }
            }

            if need_refresh {
                let _ = timer_refresh_tx.send(());
            }

            sleep(sleep_duration).await;
        }
    }
}
