use crate::interface::event_system::event::Event;

#[derive(Clone)]
pub struct SaveCheckPointEvent {}

impl Event for SaveCheckPointEvent {}
