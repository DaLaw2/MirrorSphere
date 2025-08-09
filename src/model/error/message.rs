use uuid::Uuid;
use crate::interface::actor::message::Message;
use crate::model::error::Error;

pub enum ErrorMessage {
    BackupError(Uuid, Vec<Error>),    
}

impl Message for ErrorMessage {
    type Response = ();   
}
