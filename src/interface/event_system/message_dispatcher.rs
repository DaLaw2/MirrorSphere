use crate::interface::event_system::event::Event;

pub trait MessageDispatcher<E: Event> {
    fn dispatch(&self, event: &E);
}
