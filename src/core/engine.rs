use crate::core::app_config::AppConfig;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::sync::{mpsc, oneshot, RwLock, Semaphore};

pub static ENGINE: OnceLock<RwLock<Engine>> = OnceLock::new();

type Task = Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send + 'static>;
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug)]
pub struct Engine {
    io_limit: Arc<Semaphore>,
    tx: Option<mpsc::UnboundedSender<Task>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Engine {
    pub async fn initialize() {
        let config = AppConfig::now().await;
        let io_limit = Arc::new(Semaphore::new(config.max_file_operations));
        let io_limit_clone = io_limit.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        tokio::spawn(async move {
            Self::worker_thread(io_limit_clone, rx, shutdown_rx).await;
        });
        let instance = Engine {
            io_limit,
            tx: Some(tx),
            shutdown_tx: Some(shutdown_tx),
        };
        ENGINE.set(RwLock::new(instance)).unwrap();
    }

    async fn worker_thread(
        semaphore: Arc<Semaphore>,
        mut rx: mpsc::UnboundedReceiver<Task>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) {
        loop {
            select! {
                Some(task) = rx.recv() => {
                    let io_permit = semaphore.clone();
                    tokio::spawn(async move {
                        let _io_permit = io_permit.acquire().await.unwrap();
                        task().await;
                    });
                }
                Ok(()) = &mut shutdown_rx => break,
            }
        }
    }

    pub async fn terminate() {
        let mut instance = ENGINE.get().unwrap().write().await;
        if let Some(shutdown_tx) = instance.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        instance.tx.take();
    }

    async fn submit<F, Fut>(f: F) -> anyhow::Result<()>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let instance = ENGINE.get().unwrap().read().await;
        let tx = instance.tx.as_ref().ok_or(|| {})?;
        tx.send(Box::new(move || Box::pin(f())))?;
        Ok(())
    }
}
