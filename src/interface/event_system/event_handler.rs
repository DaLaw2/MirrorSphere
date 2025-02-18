use crate::interface::event_system::event::Event;
use crate::interface::event_system::actor::Actor;

pub trait EventHandler<A: Actor, E: Event> {
    fn handle(&self, actor: &mut A, event: &E);
}

impl<A: Actor, E: Event, F: Fn(&mut A, &E)> EventHandler<A, E> for F {
    fn handle(&self, actor: &mut A, event: &E) {
        self(actor, event)
    }
}
