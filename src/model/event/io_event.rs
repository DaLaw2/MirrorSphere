use crate::interface::event_system::event::Event;

pub struct IOEvent {

}

impl Clone for IOEvent {
    fn clone(&self) -> Self {

    }
}

impl Event for IOEvent {}
