use crate::core::backup::backup_engine::BackupEngine;
use crate::core::backup::progress_tracker::ProgressTracker;
use crate::core::infrastructure::actor_system::ActorSystem;
use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::io_manager::IOManager;
use crate::interface::actor::actor::Actor;
use crate::interface::actor::message::Message;
use crate::model::core::backup::message::*;
use crate::model::error::Error;
use async_trait::async_trait;
use std::sync::Arc;

pub struct BackupService {
    backup_engine: Arc<BackupEngine>,
    progress_tracker: Arc<ProgressTracker>,
}

impl BackupService {
    pub async fn init(
        app_config: Arc<AppConfig>,
        io_manager: Arc<IOManager>,
        actor_system: Arc<ActorSystem>,
    ) {
        let progress_tracker = Arc::new(ProgressTracker::new(io_manager.clone()));
        let backup_engine = Arc::new(BackupEngine::new(
            app_config,
            io_manager,
            actor_system.clone(),
            progress_tracker.clone(),
        ));
        let backup_service = Self {
            backup_engine,
            progress_tracker,
        };
        actor_system.spawn(backup_service).await;
    }
}

#[async_trait]
impl Actor for BackupService {
    type Message = BackupServiceMessage;

    async fn pre_start(&mut self) {}

    async fn post_stop(&mut self) {
        self.backup_engine.stop_all_executions().await;
    }

    async fn receive(
        &mut self,
        message: Self::Message,
    ) -> Result<<Self::Message as Message>::Response, Error> {
        match message {
            BackupServiceMessage::ServiceCall(service_call) => match service_call {
                ServiceCallMessage::AddExecution(execution) => {
                    self.backup_engine.add_execution(execution).await;
                    Ok(BackupServiceResponse::None)
                }
                ServiceCallMessage::RemoveExecution(uuid) => {
                    self.backup_engine.resume_execution(uuid).await?;
                    Ok(BackupServiceResponse::None)
                }
                ServiceCallMessage::StartExecution(uuid) => {
                    self.backup_engine.start_execution(uuid).await?;
                    Ok(BackupServiceResponse::None)
                }
                ServiceCallMessage::SuspendExecution(uuid) => {
                    self.backup_engine.suspend_execution(uuid).await?;
                    Ok(BackupServiceResponse::None)
                }
                ServiceCallMessage::ResumeExecution(uuid) => {
                    self.backup_engine.resume_execution(uuid).await?;
                    Ok(BackupServiceResponse::None)
                }
                ServiceCallMessage::GetExecutions => {
                    let execution = self.backup_engine.get_all_executions();
                    Ok(BackupServiceResponse::ServiceCall(
                        ServiceCallResponse::GetExecutions(execution),
                    ))
                }
            },
        }
    }
}
