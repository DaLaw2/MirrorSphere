use crate::interface::actor::message::Message;
use async_trait::async_trait;

#[async_trait]
pub trait Actor: Send + 'static {
    type Message: Message;
    type Error: Send + 'static;
    async fn receive(
        &mut self,
        msg: Self::Message,
    ) -> Result<<Self::Message as Message>::Response, Self::Error>;

    async fn pre_start(&mut self);
    async fn post_stop(&mut self);
}
