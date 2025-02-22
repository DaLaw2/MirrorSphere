use crate::interface::event_system::actor::Actor;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

pub struct ActorRef<A: Actor>(Arc<Mutex<A>>);

impl<A: Actor> ActorRef<A> {
    pub fn new(actor: A) -> Self {
        Self(Arc::new(Mutex::new(actor)))
    }

    pub async fn lock(&self) -> MutexGuard<'_, A> {
        self.0.lock().await
    }
}

impl<A: Actor> Clone for ActorRef<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
