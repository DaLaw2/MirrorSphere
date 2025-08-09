use crate::interface::actor::message::Message;
use crate::model::error::Error;
use uuid::Uuid;

pub enum GuiMessage {
    ExecutionErrors(Uuid, Vec<Error>),
}

impl Message for GuiMessage {
    type Response = ();
}
