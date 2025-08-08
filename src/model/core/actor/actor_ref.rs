use crate::interface::actor::message::Message;
use crate::model::core::actor::envelope::Envelope;
use crate::model::error::actor::ActorError;
use crate::model::error::Error;
use tokio::sync::{mpsc, oneshot};

pub struct ActorRef<M: Message> {
    tx: mpsc::UnboundedSender<Envelope<M>>,
}

impl<M: Message> ActorRef<M> {
    pub fn new(tx: mpsc::UnboundedSender<Envelope<M>>) -> Self {
        Self { tx }
    }

    pub async fn tell(&self, message: M) -> Result<(), Error> {
        let envelope = Envelope::Tell(message);
        self.tx
            .send(envelope)
            .map_err(ActorError::SendMessageError)?;
        Ok(())
    }

    pub async fn ask(&self, message: M) -> Result<M::Response, Error> {
        let (reply_tx, reply_rx) = oneshot::channel::<M::Response>();
        let envelope = Envelope::Ask {
            message,
            reply_to: reply_tx,
        };
        self.tx
            .send(envelope)
            .map_err(ActorError::SendMessageError)?;
        let reply = reply_rx.await.map_err(ActorError::ActorNotResponding)?;
        Ok(reply)
    }
}

impl<M: Message> Clone for ActorRef<M> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}
