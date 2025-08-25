use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::oneshot;

#[async_trait]
pub trait Runnable: 'static {
    async fn run(self: Arc<Self>) -> oneshot::Sender<()> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(self.run_impl(shutdown_rx));

        shutdown_tx
    }

    async fn run_impl(self: Arc<Self>, shutdown_rx: oneshot::Receiver<()>);
}
