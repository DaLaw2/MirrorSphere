use crate::core::app_config::AppConfig;
use crate::core::io_manager::IOManager;
use crate::core::progress_tracker::ProgressTracker;
use crate::interface::file_system::FileSystemTrait;
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
        tokio::spawn(async move { Engine::run_backup_task(task, rx, false).await });
        Ok(())
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

    pub async fn resume_task(uuid: Uuid) -> anyhow::Result<()> {
        let instance = Self::instance().await;
        let task = instance
            .tasks
            .get(&uuid)
            .ok_or(TaskError::TaskNotFound)?
            .value()
            .clone();
        let (tx, rx) = oneshot::channel();
        instance.shutdown.insert(uuid, tx);
        tokio::spawn(async move { Engine::run_backup_task(task, rx, true).await });
        Ok(())
    }

    async fn run_backup_task(task: BackupTask, mut shutdown: OneShotReceiver<()>, resume: bool) {
        let config = AppConfig::fetch().await;

        let (mut current_level, mut errors) = if resume {
            ProgressTracker::resume_task(task.uuid).await
        } else {
            let source_root = task.source_path.clone();
            (vec![source_root], Vec::new())
        };

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

            let mut stop_flag = false;
            let results = tokio::select! {
                results = join_all(&mut worker_handles) => results,
                _ = &mut shutdown => {
                    stop_flag = true;
                    for tx in worker_shutdowns {
                        let _ = tx.send(());
                    }
                    join_all(&mut worker_handles).await
                }
            };

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

            if stop_flag {
                current_level.extend(next_level);
                ProgressTracker::save_task(task.uuid, current_level, errors).await;
                break;
            } else {
                current_level = next_level;
            }
        }
    }

    async fn worker_thread(
        worker_task: WorkerTask,
        global_queue: Arc<SegQueue<PathBuf>>,
        mut shutdown: OneShotReceiver<()>,
    ) -> (Vec<PathBuf>, Vec<anyhow::Error>) {
        let io_manager = IOManager::instance();
        let uuid = worker_task.uuid;

        let mut next_level = Vec::new();
        let mut errors = Vec::new();

        while let Some(current_dir) = global_queue.pop() {
            if shutdown.try_recv().is_ok() {
                break;
            }

            let entries = match io_manager.list_directory(uuid, current_dir.clone()).await {
                Ok(entries) => entries,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            };

            for entry in entries {
                if shutdown.try_recv().is_ok() {
                    break;
                }
                match Self::process_entry(entry, &worker_task).await {
                    Ok(Some(path)) => next_level.push(path),
                    Ok(None) => {}
                    Err(e) => errors.push(e),
                }
            }
        }

        (next_level, errors)
    }

    async fn process_entry(
        current_path: PathBuf,
        worker_task: &WorkerTask,
    ) -> anyhow::Result<Option<PathBuf>> {
        let io_manager = IOManager::instance();

        let uuid = worker_task.uuid;
        let source_root = worker_task.source_path.clone();
        let destination_root = worker_task.destination_path.clone();
        let backup_type = worker_task.backup_type;
        let comparison_mode = worker_task.comparison_mode;
        let options = worker_task.options;

        let source_path = current_path;
        let destination_path =
            Self::calculate_destination_path(&source_path, &source_root, &destination_root)?;

        let is_symlink = io_manager
            .is_symlink(source_path.clone())
            .await
            .unwrap_or(false);

        if is_symlink {
            Self::process_symlink(&source_path, &destination_path, options.follow_symlinks).await;
            return Ok(None);
        }

        let mut retval = None;

        if source_path.is_dir() {
            if !destination_path.exists() {
                io_manager
                    .create_directory(uuid, destination_path.clone())
                    .await?;
            }
            retval = Some(source_path.clone());
        } else {
            let mut lock = None;
            if options.lock_source {
                todo!()
            }

            match backup_type {
                BackupType::Full => {
                    Self::full_backup(uuid, &source_path, &destination_path).await?
                }
                BackupType::Incremental => {
                    let comparison_mode = comparison_mode.ok_or(SystemError::UnknownError)?;
                    Self::incremental_backup(uuid, &source_path, &destination_path, comparison_mode)
                        .await?
                }
            }
        }

        io_manager
            .copy_attributes(uuid, source_path.clone(), destination_path.clone())
            .await?;

        if options.backup_permission {
            io_manager
                .copy_permission(uuid, source_path, destination_path)
                .await?;
        }

        Ok(retval)
    }

    async fn process_symlink(
        source_path: &PathBuf,
        destination_path: &PathBuf,
        follow_symlink: bool,
    ) {
        todo!()
    }

    async fn full_backup(
        uuid: Uuid,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> anyhow::Result<()> {
        let io_manager = IOManager::instance();
        io_manager
            .copy_file(uuid, source_path.clone(), destination_path.clone())
            .await
    }

    async fn incremental_backup(
        uuid: Uuid,
        source_path: &PathBuf,
        destination_path: &PathBuf,
        comparison_mode: ComparisonMode,
    ) -> anyhow::Result<()> {
        let io_manager = IOManager::instance();

        let need_copy = match comparison_mode {
            ComparisonMode::Standard => {
                io_manager
                    .standard_compare(uuid, source_path.clone(), destination_path.clone())
                    .await
            }
            ComparisonMode::Advanced => {
                io_manager
                    .advance_compare(uuid, source_path.clone(), destination_path.clone())
                    .await
            }
            ComparisonMode::Thorough(hash_type) => {
                io_manager
                    .thorough_compare(
                        uuid,
                        source_path.clone(),
                        destination_path.clone(),
                        hash_type,
                    )
                    .await
            }
        }?;

        if need_copy {
            io_manager
                .copy_file(uuid, source_path.clone(), destination_path.clone())
                .await
        } else {
            Ok(())
        }
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
