use crate::core::event_system::actor_dispatcher::ActorDispatcher;
use crate::core::event_system::actor_ref::ActorRef;
use crate::interface::event_system::actor::Actor;
use crate::interface::event_system::message_dispatcher::MessageDispatcher;
use crate::interface::event_system::event::Event;
use crate::interface::event_system::event_handler::EventHandler;

pub struct ListenerGroup<E: Event> {
    dispatchers: Vec<Box<dyn MessageDispatcher<E>>>,
}

impl<E: Event> ListenerGroup<E> {
    pub fn new() -> Self {
        ListenerGroup { dispatchers: Vec::new() }
    }

    pub fn broadcast(&self, event: E) {
        for dispatcher in self.dispatchers.iter() {
            dispatcher.dispatch(&event)
        }
    }

    pub fn subscribe<A: Actor>(&mut self, actor: ActorRef<A>, handler: impl EventHandler<A, E>) {
        let handler = Box::new(handler);
        let actor_dispatcher = ActorDispatcher::new(actor, handler);
        self.dispatchers.push(Box::new(actor_dispatcher));
    }
}
