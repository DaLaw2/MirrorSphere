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
use crate::interface::communication::command::CommandHandler;
use crate::interface::communication::query::QueryHandler;
use crate::model::core::schedule::communication::{ScheduleCommand, ScheduleQuery, ScheduleQueryResponse};

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
    ) -> Result<(), Error> {
        let schedule_manager =
            Arc::new(ScheduleManager::new(database_manager).await?);
        let schedule_timer = Arc::new(ScheduleTimer::new(app_config));
        let schedule_service = Self {
            schedule_manager,
            schedule_timer,
            timer_refresh: OnceLock::new(),
            shutdowns: Vec::new(),
        };
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
    async fn process_internal_command(self: Arc<Self>, shutdown_rx: Receiver<()>) {
        todo!()
    }
}

impl CommandHandler<ScheduleCommand> for ScheduleService {
    async fn handle_command(&self, command: ScheduleCommand) -> Result<(), Error> {
        todo!()
    }
}

impl QueryHandler<ScheduleQuery> for ScheduleService {
    async fn handle_query(&self, query: ScheduleQuery) -> Result<ScheduleQueryResponse, Error> {
        todo!()
    }
}
