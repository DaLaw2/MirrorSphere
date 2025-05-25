use crate::core::app_config::AppConfig;
use crate::core::io_manager::IOManager;
use crate::interface::file_system::FileSystemTrait;
use crate::model::error::io::IOError;
use crate::model::error::system::SystemError;
use crate::model::error::task::TaskError;
use crate::model::task::{BackupTask, BackupType, ComparisonMode, WorkerTask};
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use futures::future::join_all;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
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
        let keys: Vec<Uuid> = instance
            .shutdown
            .iter()
            .map(|pair| pair.key().clone())
            .collect();
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
            .ok_or(TaskError::TaskNotFound)?
            .value()
            .clone();
        let (tx, rx) = oneshot::channel();
        instance.shutdown.insert(uuid, tx);
        tokio::spawn(async move { Engine::run_without_resume(task, rx).await });
        Ok(())
    }

    async fn run_without_resume(task: BackupTask, mut shutdown: OneShotReceiver<()>) {
        let config = AppConfig::fetch().await;
        let source_root = task.source_path.clone();
        let mut current_level = vec![source_root];
        let mut errors = Vec::new();

        while !current_level.is_empty() {
            let global_queue = Arc::new(SegQueue::new());

            for dir in current_level.clone() {
                global_queue.push(dir);
            }

            let mut worker_handles = Vec::new();
            let mut worker_shutdowns = Vec::new();

            for _ in 0..config.max_concurrency {
                let (tx, rx) = oneshot::channel();
                worker_shutdowns.push(tx);

                let queue = global_queue.clone();
                let worker_task = task.to_worker_task();

                let handle =
                    tokio::spawn(async move { Self::worker_thread(worker_task, queue, rx).await });
                worker_handles.push(handle);
            }

            tokio::select! {
                results = join_all(worker_handles) => {
                    let mut next_level = Vec::new();

                    for result in results {
                        match result {
                            Ok((subdirs, worker_errors)) => {
                                next_level.extend(subdirs);
                                errors.extend(worker_errors);
                            }
                            Err(_) => SystemError::ThreadPanic.log(),
                        }
                    }

                    current_level = next_level;
                },
                _ = &mut shutdown => {
                    for tx in worker_shutdowns {
                        let _ = tx.send(());
                    }
                }
            }
        }
    }

    pub async fn suspend_task(uuid: Uuid) -> anyhow::Result<()> {
        let instance = Self::instance().await;
        let (_, shutdown) = instance
            .shutdown
            .remove(&uuid)
            .ok_or(TaskError::TaskNotFound)?;
        let _ = shutdown.send(()).map_err(|_| TaskError::StopTaskFailed)?;
        Ok(())
    }

    pub async fn resume_task(uuid: Uuid) {}

    async fn run_with_resume(task: BackupTask, shutdown: OneShotSender<()>) {}

    async fn worker_thread(
        worker_task: WorkerTask,
        global_queue: Arc<SegQueue<PathBuf>>,
        mut shutdown: OneShotReceiver<()>,
    ) -> (Vec<PathBuf>, Vec<IOError>) {
        let io_manager = IOManager::instance();

        let uuid = worker_task.uuid;
        let backup_type = worker_task.backup_type;
        let source_root = worker_task.source_path;
        let destination_root = worker_task.destination_path;
        let comparison_mode = worker_task.comparison_mode;
        let option = worker_task.options;

        let mut next_level = Vec::new();
        let mut errors = Vec::new();

        while let Some(current_dir) = global_queue.pop() {
            if shutdown.try_recv().is_ok() {
                break;
            }

            let entries = match io_manager.list_directory(uuid, current_dir.clone()).await {
                Ok(entries) => entries,
                Err(_) => {
                    errors.push(IOError::ReadDirectoryFailed);
                    continue;
                }
            };

            for entry_path in entries {
                let mut need_copy = true;
                let source_path = entry_path;
                let destination_path = match Self::calculate_destination_path(
                    &source_path,
                    &source_root,
                    &destination_root,
                ) {
                    Ok(path) => path,
                    Err(_) => {
                        errors.push(IOError::ReadDirectoryFailed);
                        continue;
                    }
                };

                if shutdown.try_recv().is_ok() {
                    break;
                }

                if backup_type == BackupType::Incremental {
                    match comparison_mode {
                        Some(ComparisonMode::Standard) => {
                            let metadata = io_manager.get_attributes()
                        }
                        Some(ComparisonMode::Thorough(hash_type)) => {}
                        None => {}
                    }
                }
            }
        }

        (next_level, errors)
    }

    fn calculate_destination_path(
        source_path: &PathBuf,
        source_root: &PathBuf,
        destination_root: &PathBuf,
    ) -> anyhow::Result<PathBuf> {
        let relative_path = source_path
            .strip_prefix(source_root)
            .map_err(|_| SystemError::UnknownError)?;
        Ok(destination_root.join(relative_path))
    }
}
