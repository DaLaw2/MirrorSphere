use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::communication_manager::CommunicationManager;
use crate::core::infrastructure::database_manager::DatabaseManager;
use crate::core::schedule::schedule_manager::ScheduleManager;
use crate::core::schedule::schedule_timer::ScheduleTimer;
use crate::interface::core::service::Service;
use crate::model::error::Error;
use async_trait::async_trait;
use std::any::Any;
use std::sync::{Arc, OnceLock};
use tokio::sync::oneshot::Receiver;
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
impl Service for ScheduleService {
    async fn run_message_loop(self: Arc<Self>, shutdown_rx: Receiver<()>) {
        todo!()
    }
}

#[async_trait]
impl CommunicationCapable for ScheduleService {
    fn register_handlers(&self, comm: &CommunicationManager) {
        todo!()
    }

    async fn handle_command(&self, command: Box<dyn Any + Send>) -> Result<(), Error> {
        todo!()
    }

    async fn handle_query(&self, query: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>, Error> {
        todo!()
    }
}
