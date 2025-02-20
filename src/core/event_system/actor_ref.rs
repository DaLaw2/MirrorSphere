use std::sync::{Arc, LockResult, Mutex, MutexGuard};
use crate::interface::event_system::actor::Actor;

pub struct ActorRef<A: Actor>(Arc<Mutex<A>>);

impl<A: Actor> ActorRef<A> {
    pub fn new(actor: A) -> Self {
        Self(Arc::new(Mutex::new(actor)))
    }

    pub fn lock(&self) -> LockResult<MutexGuard<'_, A>> {
        self.0.lock()
    }
}

impl<A: Actor> Clone for ActorRef<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
