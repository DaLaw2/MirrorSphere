use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::communication_manager::CommunicationManager;
use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::core::schedule::schedule_manager::ScheduleManager;
use crate::core::schedule::schedule_timer::ScheduleTimer;
use crate::interface::core::runnable::Runnable;
use crate::model::error::Error;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;

pub struct ScheduleService {
    schedule_manager: Arc<ScheduleManager>,
    schedule_timer: Arc<ScheduleTimer>,
}

impl ScheduleService {
    pub async fn new(
        app_config: Arc<AppConfig>,
        database_manager: Arc<DatabaseManager>,
        communication_manager: Arc<CommunicationManager>,
    ) -> Result<Self, Error> {
        let schedule_manager =
            Arc::new(ScheduleManager::new(database_manager, communication_manager.clone()).await?);
        let schedule_timer = Arc::new(ScheduleTimer::new(app_config, communication_manager));
        let schedule_service = Self {
            schedule_manager,
            schedule_timer,
        };
        Ok(schedule_service)
    }

    pub async fn register_services(&self) {
        let schedule_manager = self.schedule_manager.clone();
        let schedule_timer = self.schedule_timer.clone();
        schedule_manager.register_services().await;
        schedule_timer.register_services().await;
    }
}

#[async_trait]
impl Runnable for ScheduleService {
    async fn run_impl(self: Arc<Self>, shutdown_rx: Receiver<()>) {
        let schedule_timer = self.schedule_timer.clone();
        let timer_shutdown = schedule_timer.run().await;
        let _ = shutdown_rx.await;
        let _ = timer_shutdown.send(());
    }
}
