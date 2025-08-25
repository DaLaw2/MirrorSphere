use crate::core::infrastructure::app_config::AppConfig;
use crate::interface::communication::command::*;
use crate::interface::communication::event::Event;
use crate::interface::communication::event::EventBroadcaster;
use crate::interface::communication::query::*;
use crate::model::core::infrastructure::event_broadcaster::TypedEventBroadcaster;
use crate::model::error::misc::MiscError;
use crate::model::error::Error;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct CommunicationManager {
    app_config: Arc<AppConfig>,
    command_handlers: DashMap<TypeId, CommandHandlerFn>,
    query_handlers: DashMap<TypeId, QueryHandlerFn>,
    event_broadcasters: DashMap<TypeId, Box<dyn EventBroadcaster>>,
}

impl CommunicationManager {
    pub fn new(app_config: Arc<AppConfig>) -> Self {
        Self {
            app_config,
            command_handlers: DashMap::new(),
            query_handlers: DashMap::new(),
            event_broadcasters: DashMap::new(),
        }
    }

    pub fn with_service<S: Send + Sync + 'static>(
        self: Arc<Self>,
        service: Arc<S>,
    ) -> ServiceRegistrar<S> {
        ServiceRegistrar::new(service, self)
    }

    pub fn register_command_handler<C: Command + 'static>(
        &self,
        handler: Arc<dyn CommandHandler<C> + Send + Sync>,
    ) {
        let type_id = TypeId::of::<C>();
        let boxed_handler: CommandHandlerFn = Box::new(move |command: Box<dyn Any + Send>| {
            let handler = handler.clone();
            Box::pin(async move {
                let command = *command
                    .downcast::<C>()
                    .map_err(|_| MiscError::TypeMismatch)?;
                handler.handle_command(command).await
            }) as CommandFuture
        });

        self.command_handlers.insert(type_id, boxed_handler);
    }

    pub async fn send_command<C: Command + 'static>(&self, command: C) -> Result<(), Error> {
        let type_id = TypeId::of::<C>();
        if let Some(handler) = self.command_handlers.get(&type_id) {
            handler(Box::new(command)).await
        } else {
            Err(MiscError::HandlerNotFound)?
        }
    }

    pub fn register_query_handler<Q: Query + 'static>(
        &self,
        handler: Arc<dyn QueryHandler<Q> + Send + Sync>,
    ) {
        let type_id = TypeId::of::<Q>();
        let boxed_handler: QueryHandlerFn = Box::new(move |query: Box<dyn Any + Send>| {
            let handler = handler.clone();
            Box::pin(async move {
                let query = *query.downcast::<Q>().map_err(|_| MiscError::TypeMismatch)?;
                let response = handler.handle_query(query).await?;
                Ok(Box::new(response) as Box<dyn Any + Send>)
            }) as QueryFuture
        });

        self.query_handlers.insert(type_id, boxed_handler);
    }

    pub async fn send_query<Q: Query + 'static>(&self, query: Q) -> Result<Q::Response, Error> {
        let type_id = TypeId::of::<Q>();
        if let Some(handler) = self.query_handlers.get(&type_id) {
            let response = handler(Box::new(query)).await?;
            Ok(*response
                .downcast::<Q::Response>()
                .map_err(|_| MiscError::TypeMismatch)?)
        } else {
            Err(MiscError::HandlerNotFound)?
        }
    }

    pub fn register_event_type<E: Event + 'static>(&self) {
        let channel_capacity = self.app_config.channel_capacity;
        let type_id = TypeId::of::<E>();
        let (tx, _) = broadcast::channel::<E>(channel_capacity);
        let broadcaster = TypedEventBroadcaster { sender: tx };
        self.event_broadcasters
            .insert(type_id, Box::new(broadcaster));
    }

    pub fn subscribe_event<E: Event + 'static>(&self) -> Result<broadcast::Receiver<E>, Error> {
        let type_id = TypeId::of::<E>();
        let broadcaster = self
            .event_broadcasters
            .get(&type_id)
            .ok_or(MiscError::TypeNotRegistered)?;
        let receiver_box = broadcaster.subscribe_typed();
        let receiver = *receiver_box
            .downcast::<broadcast::Receiver<E>>()
            .map_err(|_| MiscError::TypeMismatch)?;
        Ok(receiver)
    }

    pub async fn publish_event<E: Event + 'static>(&self, event: E) -> Result<(), Error> {
        let type_id = TypeId::of::<E>();
        let broadcaster = self
            .event_broadcasters
            .get(&type_id)
            .ok_or(MiscError::TypeNotRegistered)?;
        broadcaster.broadcast_event(Box::new(event))
    }
}

pub struct ServiceRegistrar<S> {
    service: Arc<S>,
    comm: Arc<CommunicationManager>,
}

impl<S: Send + Sync + 'static> ServiceRegistrar<S> {
    fn new(service: Arc<S>, comm: Arc<CommunicationManager>) -> Self {
        Self { service, comm }
    }

    pub fn command<C: Command + 'static>(self) -> Self
    where
        S: CommandHandler<C>,
    {
        let handler: Arc<dyn CommandHandler<C> + Send + Sync> = self.service.clone();
        self.comm.register_command_handler::<C>(handler);
        self
    }

    pub fn query<Q: Query + 'static>(self) -> Self
    where
        S: QueryHandler<Q>,
    {
        let handler: Arc<dyn QueryHandler<Q> + Send + Sync> = self.service.clone();
        self.comm.register_query_handler::<Q>(handler);
        self
    }

    pub fn event<E: Event + 'static>(self) -> Self {
        self.comm.register_event_type::<E>();
        self
    }

    pub fn build(self) -> Arc<CommunicationManager> {
        self.comm
    }
}
