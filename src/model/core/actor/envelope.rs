use crate::interface::actor::message::Message;
use tokio::sync::oneshot;

pub enum Envelope<M: Message> {
    Tell(M),
    Ask {
        message: M,
        reply_to: oneshot::Sender<M::Response>,
    }
}
