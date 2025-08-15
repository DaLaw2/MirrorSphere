use crate::interface::actor::message::Message;
use crate::model::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait Actor: Send + 'static {
    type Message: Message;
    async fn pre_start(&mut self);
    async fn post_stop(&mut self);
    async fn receive(
        &mut self,
        message: Self::Message,
    ) -> Result<<Self::Message as Message>::Response, Error>;
}
