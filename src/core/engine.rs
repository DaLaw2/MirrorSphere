use std::collections::VecDeque;
use crate::model::backup_task::BackupTask;
use crate::utils::log_entry::task::TaskEntry;
use dashmap::DashMap;
use std::sync::OnceLock;
use tokio::fs::File;
use tokio::sync::oneshot::Sender;
use uuid::Uuid;

pub static ENGINE: OnceLock<Engine> = OnceLock::new();

#[derive(Debug)]
pub struct Engine {
    tasks: DashMap<Uuid, BackupTask>,
    shutdown: DashMap<Uuid, Sender<()>>,
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
        let keys: Vec<Uuid> = instance.shutdown.iter().map(|pair| pair.key()).collect();
        for uuid in keys {
            if let Some((_, sender)) = instance.shutdown.remove(&uuid) {
                let _ = sender.send(());
            }
        }
    }

    pub async fn start_task(uuid: Uuid) -> anyhow::Result<()> {
        let instance = Self::instance().await;
        let task = instance
            .tasks
            .get(&uuid)
            .ok_or(TaskEntry::TaskNotFound)?
            .value()
            .clone();
        tokio::spawn(async move { Engine::run_without_resume(task).await });
        Ok(())
    }

    async fn run_without_resume(task: BackupTask) {

    }

    pub async fn suspend_task(uuid: Uuid) {}

    pub async fn resume_task(uuid: Uuid) {}

    async fn run_with_resume(task: BackupTask) {

    }
}
