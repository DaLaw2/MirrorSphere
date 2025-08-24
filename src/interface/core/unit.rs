use crate::model::error::Error;
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedReceiver;

#[async_trait]
pub trait Unit {
    type Command: Send + 'static;
    type InternalCommand: Send + 'static;
    type Query: Send + 'static;
    type QueryResponse: Send + 'static;

    fn get_internal_channel(&self) -> UnboundedReceiver<Self::InternalCommand>;
    async fn handle_command(&self, command: Self::Command) -> Result<(), Error>;
    async fn handle_internal_command(&self, command: Self::InternalCommand) -> Result<(), Error>;
    async fn handle_query(&self, query: Self::Query) -> Result<Self::QueryResponse, Error>;
}
