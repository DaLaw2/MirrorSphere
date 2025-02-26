use crate::interface::event_system::event::Event;
use async_trait::async_trait;

#[async_trait]
pub trait Dispatcher<E: Event> {
    async fn dispatch(&self, event: E);
}
