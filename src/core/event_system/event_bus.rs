use crate::core::event_system::actor_ref::ActorRef;
use crate::core::event_system::listener_group::ListenerGroup;
use crate::interface::event_system::actor::Actor;
use crate::interface::event_system::event::Event;
use crate::interface::event_system::event_handler::EventHandler;
use crate::model::error::system::SystemError;
use crate::model::error::Error;
use dashmap::DashMap;
use std::any::{Any, TypeId};

pub struct EventBus {
    listeners: DashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: DashMap::new(),
        }
    }

    pub async fn subscribe<A: Actor, E: Event>(
        &self,
        actor: &ActorRef<A>,
        handler: impl EventHandler<A, E> + Send + Sync + 'static,
    ) -> Result<(), Error> {
        let type_id = TypeId::of::<ListenerGroup<E>>();
        let mut entry = self
            .listeners
            .entry(type_id)
            .or_insert_with(|| Box::new(ListenerGroup::<E>::new()));
        let listeners = entry
            .value_mut()
            .downcast_mut::<ListenerGroup<E>>()
            .ok_or(SystemError::InternalError)?;
        listeners.subscribe(actor.clone(), handler);
        Ok(())
    }

    pub async fn publish<E: Event>(&self, event: E) -> Result<(), Error> {
        let type_id = TypeId::of::<ListenerGroup<E>>();
        if let Some(listeners) = self.listeners.get_mut(&type_id) {
            let listeners = listeners
                .value()
                .downcast_ref::<ListenerGroup<E>>()
                .ok_or(SystemError::InternalError)?;
            listeners.broadcast(event).await;
        }
        Ok(())
    }
}
