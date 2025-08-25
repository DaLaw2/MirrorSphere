use crate::interface::communication::event::Event;
use crate::interface::communication::event::EventBroadcaster;
use crate::model::error::misc::MiscError;
use crate::model::error::Error;
use std::any::Any;
use tokio::sync::broadcast;

pub struct TypedEventBroadcaster<E: Event> {
    pub sender: broadcast::Sender<E>,
}

impl<E: Event + 'static> EventBroadcaster for TypedEventBroadcaster<E> {
    fn subscribe_typed(&self) -> Box<dyn Any + Send> {
        Box::new(self.sender.subscribe())
    }

    fn broadcast_event(&self, event: Box<dyn Any + Send>) -> Result<(), Error> {
        let typed_event = *event.downcast::<E>().map_err(|_| MiscError::TypeMismatch)?;
        let _ = self.sender.send(typed_event);
        Ok(())
    }
}
