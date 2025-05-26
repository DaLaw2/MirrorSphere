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
use std::collections::{HashSet, VecDeque};
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
                if let Err(_) = sender.send(()) {
                    TaskError::StopTaskFailed.log();
                }
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
                    Ok((worker_next_level, worker_errors)) => {
                        next_level.extend(worker_next_level);
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

        let mirror = worker_task.options.mirror;

        let mut next_level = Vec::new();
        let mut errors = Vec::new();

        while let Some(current_dir) = global_queue.pop() {
            if shutdown.try_recv().is_ok() {
                break;
            }

            let entries = match io_manager.list_directory(&current_dir).await {
                Ok(entries) => entries,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            };

            for entry in entries.iter() {
                if shutdown.try_recv().is_ok() {
                    break;
                }
                match Self::process_entry(entry, &worker_task).await {
                    Ok(Some(path)) => next_level.push(path),
                    Ok(None) => {}
                    Err(e) => errors.push(e),
                }
            }

            if mirror {
                let source_entries = entries;
                let destination_dir = match Self::calculate_destination_path(
                    &current_dir,
                    &worker_task.source_path,
                    &worker_task.destination_path,
                ) {
                    Ok(dir) => dir,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };
                if destination_dir.exists() {
                    match io_manager.list_directory(&destination_dir).await {
                        Ok(destination_entries) => {
                            let (_, mirror_errors) =
                                Self::mirror_cleanup(source_entries, destination_entries).await;
                            errors.extend(mirror_errors);
                        }
                        Err(e) => errors.push(e),
                    }
                }
            }
        }

        (next_level, errors)
    }

    async fn process_entry(
        current_path: &PathBuf,
        worker_task: &WorkerTask,
    ) -> anyhow::Result<Option<PathBuf>> {
        let io_manager = IOManager::instance();

        let source_root = &worker_task.source_path;
        let destination_root = &worker_task.destination_path;
        let options = worker_task.options;

        let source_path = current_path.clone();
        let destination_path =
            Self::calculate_destination_path(&source_path, &source_root, &destination_root)?;

        let is_symlink = io_manager.is_symlink(&source_path).await.unwrap_or(false);

        if is_symlink {
            Self::process_symlink(&source_path, &destination_path, options).await?;
            return Ok(None);
        }

        if source_path.is_dir() {
            Self::backup_directory(&source_path, &destination_path, worker_task).await
        } else {
            Self::backup_file(&source_path, &destination_path, worker_task).await
        }
    }

    async fn backup_directory(
        source_path: &PathBuf,
        destination_path: &PathBuf,
        worker_task: &WorkerTask,
    ) -> anyhow::Result<Option<PathBuf>> {
        let io_manager = IOManager::instance();

        if !destination_path.exists() {
            io_manager.create_directory(&destination_path).await?;
        }

        io_manager
            .copy_attributes(source_path, destination_path)
            .await?;

        if worker_task.options.backup_permission {
            io_manager
                .copy_permission(source_path, destination_path)
                .await?;
        }

        Ok(Some(source_path.clone()))
    }

    async fn backup_file(
        source_path: &PathBuf,
        destination_path: &PathBuf,
        worker_task: &WorkerTask,
    ) -> anyhow::Result<Option<PathBuf>> {
        let io_manager = IOManager::instance();

        #[allow(unused_variables)]
        let mut file_lock = None;
        #[allow(unused_assignments)]
        if worker_task.options.lock_source {
            file_lock = Some(io_manager.acquire_file_lock(source_path).await?);
        }

        match worker_task.backup_type {
            BackupType::Full => Self::full_backup(source_path, destination_path).await?,
            BackupType::Incremental => {
                let comparison_mode = worker_task
                    .comparison_mode
                    .ok_or(SystemError::UnknownError)?;
                Self::incremental_backup(source_path, destination_path, comparison_mode).await?
            }
        }

        io_manager
            .copy_attributes(source_path, destination_path)
            .await?;

        if worker_task.options.backup_permission {
            io_manager
                .copy_permission(source_path, destination_path)
                .await?;
        }

        drop(file_lock);

        Ok(None)
    }

    #[inline(always)]
    async fn process_symlink(
        source_path: &PathBuf,
        destination_path: &PathBuf,
        worker_task: &WorkerTask,
    ) -> anyhow::Result<()> {
        if worker_task.options.follow_symlinks {
            Self::follow_symlink(source_path, destination_path, worker_task).await
        } else {
            Self::copy_symlink(source_path, destination_path, worker_task).await
        }
    }

    async fn follow_symlink(
        source_path: &PathBuf,
        destination_path: &PathBuf,
        worker_task: &WorkerTask,
    ) -> anyhow::Result<()> {
        let io_manager = IOManager::instance();

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((source_path.clone(), destination_path.clone()));

        while let Some((current_source, current_dest)) = queue.pop_front() {
            let is_symlink = io_manager
                .is_symlink(&current_source)
                .await
                .unwrap_or(false);
            let canonical_path = if is_symlink {
                match current_source.canonicalize() {
                    Ok(path) => path,
                    Err(_) => continue,
                }
            } else {
                current_source.clone()
            };

            if visited.contains(&canonical_path) {
                continue;
            }
            visited.insert(canonical_path.clone());

            if canonical_path.is_dir() {
                Self::backup_directory(&canonical_path, &current_dest, worker_task).await?;

                let entries = io_manager.list_directory(&canonical_path).await?;
                for entry in entries {
                    let relative_path = match entry.strip_prefix(&canonical_path) {
                        Ok(rel_path) => rel_path.to_path_buf(),
                        Err(_) => match entry.file_name() {
                            Some(name) => PathBuf::from(name),
                            None => continue,
                        }
                    };
                    let new_destination = current_dest.join(relative_path);
                    queue.push_back((entry, new_destination));
                }
            } else {
                Self::backup_file(&canonical_path, &current_dest, worker_task).await?;
            }
        }

        Ok(())
    }

    async fn copy_symlink(
        source_path: &PathBuf,
        destination_path: &PathBuf,
        worker_task: &WorkerTask,
    ) -> anyhow::Result<()> {
        let io_manager = IOManager::instance();

        io_manager
            .copy_symlink(source_path, destination_path)
            .await?;

        io_manager
            .copy_attributes(source_path, destination_path)
            .await?;

        if worker_task.options.backup_permission {
            io_manager
                .copy_permission(source_path, destination_path)
                .await?;
        }

        Ok(())
    }

    #[inline(always)]
    async fn full_backup(source_path: &PathBuf, destination_path: &PathBuf) -> anyhow::Result<()> {
        let io_manager = IOManager::instance();
        io_manager.copy_file(source_path, destination_path).await
    }

    async fn incremental_backup(
        source_path: &PathBuf,
        destination_path: &PathBuf,
        comparison_mode: ComparisonMode,
    ) -> anyhow::Result<()> {
        let io_manager = IOManager::instance();

        let need_copy = !match comparison_mode {
            ComparisonMode::Standard => {
                io_manager
                    .standard_compare(source_path, destination_path)
                    .await
            }
            ComparisonMode::Advanced => {
                io_manager
                    .advance_compare(source_path, destination_path)
                    .await
            }
            ComparisonMode::Thorough(hash_type) => {
                io_manager
                    .thorough_compare(source_path, destination_path, hash_type)
                    .await
            }
        }?;

        if need_copy {
            io_manager.copy_file(source_path, destination_path).await
        } else {
            Ok(())
        }
    }

    async fn mirror_cleanup(
        source_entries: Vec<PathBuf>,
        destination_entries: Vec<PathBuf>,
    ) -> ((), Vec<anyhow::Error>) {
        let io_manager = IOManager::instance();

        let mut errors = Vec::new();

        let source_names: HashSet<_> = source_entries
            .into_iter()
            .filter_map(|path| path.file_name().map(|name| name.to_owned()))
            .collect();

        for dest_entry in destination_entries {
            if let Some(file_name) = dest_entry.file_name() {
                if !source_names.contains(file_name) {
                    if dest_entry.is_dir() {
                        if let Err(e) = io_manager.delete_directory(&dest_entry).await {
                            errors.push(e);
                        }
                    } else {
                        if let Err(e) = io_manager.delete_file(&dest_entry).await {
                            errors.push(e);
                        }
                    }
                }
            }
        }

        ((), errors)
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
