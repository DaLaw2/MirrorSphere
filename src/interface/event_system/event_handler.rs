use async_trait::async_trait;
use futures::future::BoxFuture;
use crate::interface::event_system::event::Event;
use crate::interface::event_system::actor::Actor;

#[async_trait]
pub trait EventHandler<A: Actor, E: Event> {
    async fn handle(&self, actor: &mut A, event: E);
}

#[async_trait]
impl<A: Actor, E: Event, F> EventHandler<A, E> for F
where
    F: for<'a> Fn(&'a mut A, E) -> BoxFuture<'a, ()> + Send + Sync + 'static,
{
    async fn handle(&self, actor: &mut A, event: E) {
        self(actor, event).await
    }
}
