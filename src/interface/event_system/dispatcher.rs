use crate::interface::event_system::event::Event;

pub trait Dispatcher<E: Event> {
    fn dispatch(&self, event: &E);
}
