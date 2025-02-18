use crate::core::event_system::actor_ref::ActorRef;
use crate::interface::event_system::actor::Actor;
use crate::interface::event_system::event::Event;
use crate::interface::event_system::event_handler::EventHandler;
use crate::interface::event_system::message_dispatcher::MessageDispatcher;

pub struct ActorDispatcher<A: Actor, E: Event> {
    actor: ActorRef<A>,
    handler: Box<dyn EventHandler<A, E>>,
}

impl<A: Actor, E: Event> ActorDispatcher<A, E> {
    pub fn new(actor: ActorRef<A>, handler: Box<dyn EventHandler<A, E>>) -> Self {
        Self { actor, handler }
    }
}

impl<A: Actor, E: Event> MessageDispatcher<E> for ActorDispatcher<A, E> {
    fn dispatch(&self, event: &E) {
        let mut actor = self.actor.lock().unwrap();
        self.handler.handle(&mut actor, event);
    }
}
