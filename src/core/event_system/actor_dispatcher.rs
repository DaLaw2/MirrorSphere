use crate::core::event_system::actor_ref::ActorRef;
use crate::interface::event_system::actor::Actor;
use crate::interface::event_system::event::Event;
use crate::interface::event_system::event_handler::EventHandler;
use crate::interface::event_system::dispatcher::Dispatcher;
use crate::interface::ThreadSafe;

pub struct ActorDispatcher<A: Actor, E: Event> {
    actor: ActorRef<A>,
    handler: Box<dyn EventHandler<A, E> + ThreadSafe>,
}

impl<A: Actor, E: Event> ActorDispatcher<A, E> {
    pub fn new(actor: ActorRef<A>, handler: Box<dyn EventHandler<A, E> + ThreadSafe>) -> Self {
        Self { actor, handler }
    }
}

impl<A: Actor, E: Event> Dispatcher<E> for ActorDispatcher<A, E> {
    fn dispatch(&self, event: &E) {
        let mut actor = self.actor.lock().unwrap();
        self.handler.handle(&mut actor, event);
    }
}
