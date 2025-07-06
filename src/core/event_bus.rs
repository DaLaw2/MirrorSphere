use crate::interface::event::Event;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::mpsc::{channel, Receiver};

pub struct EventBus {
    channels: DashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            channels: DashMap::new(),
        }
    }

    pub fn subscribe<E: Event>(&self) -> Receiver<E> {
        let (tx, rx) = channel();
        let type_id = TypeId::of::<E>();

        let handler = Box::new(move |event: &dyn Any| {
            if let Some(typed_event) = event.downcast_ref::<E>() {
                let _ = tx.send(typed_event.clone());
            }
        });

        self.channels
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(handler);

        rx
    }

    pub fn publish<E: Event>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        if let Some(handlers) = self.channels.get(&type_id) {
            for handler in handlers.value() {
                handler(&event);
            }
        }
    }
}
