use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::oneshot;

#[async_trait]
pub trait Service {
    async fn run(self: Arc<Self>) -> oneshot::Sender<()> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(self.run_impl(shutdown_rx));

        shutdown_tx
    }

    async fn process_internal_command(self: Arc<Self>, shutdown_rx: oneshot::Receiver<()>);
}
