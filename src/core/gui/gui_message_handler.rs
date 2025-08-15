use crate::interface::actor::actor::Actor;
use crate::interface::actor::message::Message;
use crate::model::core::gui::message::GuiMessage;
use crate::model::error::Error;
use async_trait::async_trait;
use std::sync::mpsc;

pub struct GuiMessageHandler {
    subscriber: Vec<mpsc::Sender<GuiMessage>>,
}

impl GuiMessageHandler {
    pub fn new() -> Self {
        Self {
            subscriber: Vec::new(),
        }
    }

    pub fn subscribe(&mut self) -> mpsc::Receiver<GuiMessage> {
        let (tx, rx) = mpsc::channel();
        self.subscriber.push(tx);
        rx
    }

    fn broadcast(&mut self, message: GuiMessage) {
        self.subscriber
            .retain(|tx| tx.send(message.clone()).is_ok());
    }
}

#[async_trait]
impl Actor for GuiMessageHandler {
    type Message = GuiMessage;

    async fn pre_start(&mut self) {}

    async fn post_stop(&mut self) {
        self.subscriber.clear();
    }

    async fn receive(
        &mut self,
        message: Self::Message,
    ) -> Result<<Self::Message as Message>::Response, Error> {
        self.broadcast(message);
        Ok(())
    }
}
