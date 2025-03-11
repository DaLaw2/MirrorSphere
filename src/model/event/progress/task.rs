use crate::interface::event_system::event::Event;

#[derive(Clone)]
pub struct UpdateProgressEvent {}

impl Event for UpdateProgressEvent {}
