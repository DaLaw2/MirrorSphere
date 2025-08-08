use crate::interface::actor::actor::Actor;
use crate::model::core::actor::actor_ref::ActorRef;
use crate::model::core::actor::actor_runtime::ActorRuntime;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::mem;
use tokio::sync::oneshot;

pub struct ActorSystem {
    actors: DashMap<TypeId, Box<dyn Any + Send>>,
    shutdowns: SegQueue<oneshot::Sender<()>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {
            actors: DashMap::new(),
            shutdowns: SegQueue::new(),
        }
    }

    pub async fn spawn<A>(&mut self, actor: A)
    where
        A: Actor + 'static,
    {
        let actor_id = TypeId::of::<A>();
        let (actor_runtime, actor_ref) = ActorRuntime::new(actor);
        let shutdown = actor_runtime.run().await;
        self.actors.insert(actor_id, Box::new(actor_ref));
        self.shutdowns.push(shutdown);
    }

    pub fn shutdown(&mut self) {
        let shutdowns = mem::take(&mut self.shutdowns);
        for shutdown in shutdowns {
            let _ = shutdown.send(());
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
