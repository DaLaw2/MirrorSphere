use crate::core::event_system::actor_ref::ActorRef;
use crate::interface::event_system::actor::Actor;
use crate::interface::event_system::dispatcher::Dispatcher;
use crate::interface::event_system::event::Event;
use crate::interface::event_system::event_handler::EventHandler;
use async_trait::async_trait;

pub struct ActorDispatcher<A: Actor, E: Event> {
    actor: ActorRef<A>,
    handler: Box<dyn EventHandler<A, E> + Send + Sync + 'static>,
}

impl<A: Actor, E: Event> ActorDispatcher<A, E> {
    pub fn new(
        actor: ActorRef<A>,
        handler: Box<dyn EventHandler<A, E> + Send + Sync + 'static>,
    ) -> Self {
        Self { actor, handler }
    }
}

#[async_trait]
impl<A: Actor, E: Event> Dispatcher<E> for ActorDispatcher<A, E> {
    async fn dispatch(&self, event: E) {
        let mut actor = self.actor.lock().await;
        self.handler.handle(&mut actor, event).await;
    }
}
