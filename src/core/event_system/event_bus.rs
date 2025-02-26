use crate::core::event_system::actor_ref::ActorRef;
use crate::core::event_system::listener_group::ListenerGroup;
use crate::interface::event_system::actor::Actor;
use crate::interface::event_system::event::Event;
use crate::interface::event_system::event_handler::EventHandler;
use crate::interface::ThreadSafe;
use crate::utils::log_entry::system::SystemEntry;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::OnceLock;

static EVENT_BUS: OnceLock<EventBus> = OnceLock::new();

pub struct EventBus {
    listeners: DashMap<TypeId, Box<dyn Any + ThreadSafe>>,
}

impl EventBus {
    fn new() -> Self {
        Self {
            listeners: DashMap::new(),
        }
    }

    fn instance() -> &'static EventBus {
        &EVENT_BUS.get_or_init(|| EventBus::new())
    }

    pub async fn subscribe<A: Actor, E: Event>(
        actor: &ActorRef<A>,
        handler: impl EventHandler<A, E> + ThreadSafe,
    ) -> anyhow::Result<()> {
        let instance = Self::instance();
        let type_id = TypeId::of::<ListenerGroup<E>>();
        let mut entry = instance
            .listeners
            .entry(type_id)
            .or_insert_with(|| Box::new(ListenerGroup::<E>::new()));
        let listeners = entry
            .value_mut()
            .downcast_mut::<ListenerGroup<E>>()
            .ok_or(SystemEntry::InternalError)?;
        listeners.subscribe(actor.clone(), handler);
        Ok(())
    }

    pub async fn publish<E: Event>(event: E) -> anyhow::Result<()> {
        let instance = Self::instance();
        let type_id = TypeId::of::<ListenerGroup<E>>();
        if let Some(listeners) = instance.listeners.get_mut(&type_id) {
            let listeners = listeners
                .value()
                .downcast_ref::<ListenerGroup<E>>()
                .ok_or(SystemEntry::InternalError)?;
            listeners.broadcast(event).await;
        }
        Ok(())
    }
}
