use crate::interface::actor::actor::Actor;
use crate::model::core::actor::actor_ref::ActorRef;
use crate::model::core::actor::envelope::Envelope;
use crate::model::error::Error;
use tokio::select;
use tokio::sync::{mpsc, oneshot};

pub struct ActorRuntime<A: Actor> {
    actor: A,
    rx: mpsc::UnboundedReceiver<Envelope<A::Message>>,
}

impl<A: Actor> ActorRuntime<A> {
    pub fn new(actor: A) -> (Self, ActorRef<A::Message>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let actor_ref = ActorRef::new(tx);
        let runtime = Self { actor, rx };
        (runtime, actor_ref)
    }

    pub async fn run(mut self) -> oneshot::Sender<()> {
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        tokio::spawn(async move {
            self.actor.pre_start().await;
            loop {
                select! {
                    envelope = self.rx.recv() => {
                        match envelope {
                            Some(Envelope::Tell(message)) => {
                                let _ = self.actor.receive(message).await;
                            }
                            Some(Envelope::Ask { message, reply_to }) => {
                                if let Ok(response) = self.actor.receive(message).await {
                                    let _ = reply_to.send(response);
                                }
                            }
                            None => break,
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
            self.actor.post_stop().await;
        });
        shutdown_tx
    }
}
