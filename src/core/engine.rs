use crate::model::task::BackupTask;
use crate::utils::log_entry::task::TaskEntry;
use dashmap::DashMap;
use std::sync::OnceLock;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver as OneShotReceiver, Sender as OneShotSender};
use uuid::Uuid;

pub static ENGINE: OnceLock<Engine> = OnceLock::new();

#[derive(Debug)]
pub struct Engine {
    tasks: DashMap<Uuid, BackupTask>,
    shutdown: DashMap<Uuid, OneShotSender<()>>,
}

impl Engine {
    pub async fn initialize() {
        let instance = Engine {
            tasks: DashMap::new(),
            shutdown: DashMap::new(),
        };
        ENGINE.set(instance).unwrap();
    }

    pub async fn instance() -> &'static Engine {
        ENGINE.get().unwrap()
    }

    pub async fn terminate() {
        let instance = Self::instance().await;
        let keys: Vec<Uuid> = instance.shutdown.iter().map(|pair| pair.key().clone()).collect();
        for uuid in keys {
            if let Some((_, sender)) = instance.shutdown.remove(&uuid) {
                let _ = sender.send(());
            }
        }
    }

    pub async fn add_task(task: BackupTask) {
        let instance = Self::instance().await;
        instance.tasks.insert(task.uuid, task);
    }

    pub async fn remove_task(task: &Uuid) {
        let instance = Self::instance().await;
        instance.tasks.remove(task);
    }

    pub async fn start_task(uuid: Uuid) -> anyhow::Result<()> {
        let instance = Self::instance().await;
        let task = instance
            .tasks
            .get(&uuid)
            .ok_or(TaskEntry::TaskNotFound)?
            .value()
            .clone();
        let (tx, rx) = oneshot::channel();
        instance.shutdown.insert(uuid, tx);
        tokio::spawn(async move { Engine::run_without_resume(task, rx).await });
        Ok(())
    }

    async fn run_without_resume(task: BackupTask, shutdown: OneShotReceiver<()>) {

    }

    pub async fn suspend_task(uuid: Uuid) -> anyhow::Result<()> {
        let instance = Self::instance().await;
        let (_, channel) = instance
            .shutdown
            .remove(&uuid)
            .ok_or(TaskEntry::TaskNotFound)?;
        let _ = channel.send(());
        Ok(())
    }

    pub async fn resume_task(uuid: Uuid, shutdown: OneShotSender<()>) {

    }

    async fn run_with_resume(task: BackupTask) {}
}
