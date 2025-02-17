
use tokio::sync::mpsc;
use crate::model::event::Event;

pub struct EventBus {
    task_event: mpsc::UnboundedSender<Event>,
    error_event: mpsc::UnboundedSender<Event>,
}

impl EventBus {
    pub async fn new() -> EventBus {

    }
}
