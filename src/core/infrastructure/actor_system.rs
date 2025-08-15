use crate::interface::actor::actor::Actor;
use crate::model::core::actor::actor_ref::ActorRef;
use crate::model::core::actor::actor_runtime::ActorRuntime;
use crate::model::error::system::SystemError;
use dashmap::DashMap;
use macros::log;
use std::any::{Any, TypeId};
use tokio::sync::oneshot;

pub struct ActorSystem {
    actors: DashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
    shutdowns: DashMap<TypeId, oneshot::Sender<()>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {
            actors: DashMap::new(),
            shutdowns: DashMap::new(),
        }
    }

    pub async fn spawn<A>(&self, actor: A)
    where
        A: Actor + 'static,
    {
        let actor_id = TypeId::of::<A>();
        let (actor_runtime, actor_ref) = ActorRuntime::new(actor);
        let shutdown = actor_runtime.run().await;
        self.actors.insert(actor_id, Box::new(actor_ref));
        self.shutdowns.insert(actor_id, shutdown);
    }

    pub fn shutdown(&self) {
        let keys = self
            .shutdowns
            .iter()
            .map(|x| x.key().clone())
            .collect::<Vec<_>>();
        for key in keys {
            if let Some((_, shutdown)) = self.shutdowns.remove(&key) {
                if let Err(_) = shutdown.send(()) {
                    log!(SystemError::ShutdownSignalFailed);
                }
            }
        }
    }

    pub fn actor_of<A: Actor>(&self) -> Option<ActorRef<A::Message>> {
        let type_id = TypeId::of::<A>();
        self.actors
            .get(&type_id)?
            .downcast_ref::<ActorRef<A::Message>>()
            .cloned()
    }
}
