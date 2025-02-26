use crate::core::event_system::actor_dispatcher::ActorDispatcher;
use crate::core::event_system::actor_ref::ActorRef;
use crate::interface::event_system::actor::Actor;
use crate::interface::event_system::dispatcher::Dispatcher;
use crate::interface::event_system::event::Event;
use crate::interface::event_system::event_handler::EventHandler;
use crate::interface::ThreadSafe;
use futures::future;

pub struct ListenerGroup<E: Event> {
    dispatchers: Vec<Box<dyn Dispatcher<E> + ThreadSafe>>,
}

impl<E: Event> ListenerGroup<E> {
    pub fn new() -> Self {
        ListenerGroup {
            dispatchers: Vec::new(),
        }
    }

    pub fn subscribe<A: Actor>(
        &mut self,
        actor: ActorRef<A>,
        handler: impl EventHandler<A, E> + ThreadSafe,
    ) {
        let handler = Box::new(handler);
        let actor_dispatcher = ActorDispatcher::new(actor, handler);
        self.dispatchers.push(Box::new(actor_dispatcher));
    }

    pub async fn broadcast(&self, event: E) {
        let futures = self
            .dispatchers
            .iter()
            .map(|dispatcher| dispatcher.dispatch(event.clone()));
        future::join_all(futures).await;
    }
}
