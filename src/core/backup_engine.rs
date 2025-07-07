use crate::core::app_config::AppConfig;
use crate::core::event_bus::EventBus;
use crate::core::io_manager::IOManager;
use crate::core::progress_tracker::ProgressTracker;
use crate::interface::file_system::FileSystemTrait;
use crate::model::error::system::SystemError;
use crate::model::error::task::TaskError;
use crate::model::error::Error;
use crate::model::event::tasks::*;
use crate::model::task::{BackupState, BackupTask, BackupType, ComparisonMode};
use async_trait::async_trait;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use futures::future::join_all;
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use uuid::Uuid;
use crate::log;

pub struct BackupEngine {
    config: Arc<AppConfig>,
    event_bus: Arc<EventBus>,
    io_manager: Arc<IOManager>,
    progress_tracker: Arc<ProgressTracker>,
    tasks: Arc<DashMap<Uuid, BackupTask>>,
    running_tasks: Arc<DashMap<Uuid, (oneshot::Sender<()>, JoinHandle<()>)>>,
}

impl BackupEngine {
    pub async fn new(
        config: Arc<AppConfig>,
        event_bus: Arc<EventBus>,
        io_manager: Arc<IOManager>,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Self {
        Self {
            config,
            event_bus,
            io_manager,
            progress_tracker,
            tasks: Arc::new(DashMap::new()),
            running_tasks: Arc::new(DashMap::new()),
        }
    }

    pub async fn stop_all_tasks(&self) {
        let keys: Vec<Uuid> = self
            .running_tasks
            .iter()
            .map(|pair| pair.key().clone())
            .collect();
        for uuid in keys {
            if let Some((_, (shutdown, handle))) = self.running_tasks.remove(&uuid) {
                if shutdown.send(()).is_err() {
                    log!(TaskError::StopTaskFailed);
                    continue;
                }
                if handle.await.is_err() {
                    log!(SystemError::ThreadPanic);
                }
            }
        }
    }

    pub async fn add_task(&self, task: BackupTask) {
        self.tasks.insert(task.uuid, task);
    }

    pub async fn remove_task(&self, uuid: &Uuid) {
        self.tasks.remove(uuid);
    }

    pub async fn start_task(&self, uuid: Uuid) -> Result<(), Error> {
        if self.running_tasks.contains_key(&uuid) {
            Err(TaskError::IllegalTaskState)?
        }

        let mut ref_mut = self.tasks.get_mut(&uuid).ok_or(TaskError::TaskNotFound)?;
        let task = ref_mut.value_mut();
        if task.state != BackupState::Pending {
            Err(TaskError::IllegalTaskState)?
        }
        task.state = BackupState::Running;

        let task_runner = self.to_task_runner();
        let task = task.clone();
        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { task_runner.run(task, rx, false).await });
        self.running_tasks.insert(uuid, (tx, handle));
        Ok(())
    }

    pub async fn suspend_task(&self, uuid: Uuid) -> Result<(), Error> {
        let mut ref_mut = self.tasks.get_mut(&uuid).ok_or(TaskError::TaskNotFound)?;
        let task = ref_mut.value_mut();
        if task.state != BackupState::Running {
            Err(TaskError::IllegalTaskState)?
        }
        task.state = BackupState::Suspended;
        drop(ref_mut);

        let (_, (shutdown, handle)) = self
            .running_tasks
            .remove(&uuid)
            .ok_or(TaskError::TaskNotFound)?;
        shutdown.send(()).map_err(|_| TaskError::StopTaskFailed)?;
        handle.await.map_err(|_| SystemError::ThreadPanic)?;
        Ok(())
    }

    pub async fn resume_task(&self, uuid: Uuid) -> Result<(), Error> {
        if self.running_tasks.contains_key(&uuid) {
            Err(TaskError::IllegalTaskState)?
        }

        let mut ref_mut = self.tasks.get_mut(&uuid).ok_or(TaskError::TaskNotFound)?;
        let task = ref_mut.value_mut();
        if task.state != BackupState::Suspended {
            Err(TaskError::IllegalTaskState)?
        }
        task.state = BackupState::Running;

        let task_runner = self.to_task_runner();
        let task = task.clone();
        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { task_runner.run(task, rx, true).await });
        self.running_tasks.insert(uuid, (tx, handle));
        Ok(())
    }

    fn to_task_runner(&self) -> BackupTaskRunner {
        let config = self.config.clone();
        let io_manager = self.io_manager.clone();
        let progress_tracker = self.progress_tracker.clone();
        let tasks = self.tasks.clone();
        let running_tasks = self.running_tasks.clone();
        BackupTaskRunner::new(config, io_manager, progress_tracker, tasks, running_tasks)
    }
}

struct BackupTaskRunner {
    config: Arc<AppConfig>,
    io_manager: Arc<IOManager>,
    progress_tracker: Arc<ProgressTracker>,
    tasks: Arc<DashMap<Uuid, BackupTask>>,
    running_tasks: Arc<DashMap<Uuid, (oneshot::Sender<()>, JoinHandle<()>)>>,
}

impl BackupTaskRunner {
    pub fn new(
        config: Arc<AppConfig>,
        io_manager: Arc<IOManager>,
        progress_tracker: Arc<ProgressTracker>,
        tasks: Arc<DashMap<Uuid, BackupTask>>,
        running_tasks: Arc<DashMap<Uuid, (oneshot::Sender<()>, JoinHandle<()>)>>,
    ) -> Self {
        Self {
            config,
            io_manager,
            progress_tracker,
            tasks,
            running_tasks,
        }
    }

    async fn run(&self, task: BackupTask, mut shutdown: oneshot::Receiver<()>, resume: bool) {
        let config = &self.config;
        let progress_tracker = &self.progress_tracker;

        let (mut current_level, mut errors) = if resume {
            progress_tracker.resume_task(task.uuid).await
        } else {
            let source_root = task.source_path.clone();
            (vec![source_root], Vec::new())
        };

        let mut shutdown_flag = false;
        while !current_level.is_empty() {
            let global_queue = Arc::new(SegQueue::new());

            for dir in current_level.clone() {
                global_queue.push(dir);
            }

            let mut worker_handles = Vec::new();
            let mut worker_shutdowns = Vec::new();

            for _ in 0..config.max_concurrency {
                let worker = self.to_backup_worker();
                let (tx, rx) = oneshot::channel();
                let task = task.clone();
                let queue = global_queue.clone();
                let handle = tokio::spawn(async move { worker.run(task, queue, rx).await });
                worker_shutdowns.push(tx);
                worker_handles.push(handle);
            }

            let workers_results = tokio::select! {
                results = join_all(&mut worker_handles) => results,
                _ = &mut shutdown => {
                    shutdown_flag = true;
                    for shutdown in worker_shutdowns {
                        if shutdown.send(()).is_err() {
                            log!(TaskError::StopTaskFailed);
                        }
                    }
                    join_all(&mut worker_handles).await
                }
            };

            let mut next_level = Vec::new();
            for result in workers_results {
                match result {
                    Ok((worker_next_level, worker_errors)) => {
                        next_level.extend(worker_next_level);
                        errors.extend(worker_errors);
                    }
                    Err(_) => log!(SystemError::ThreadPanic),
                }
            }

            if shutdown_flag {
                current_level.extend(next_level);
                let _ = progress_tracker
                    .save_task(task.uuid, current_level, errors)
                    .await;
                unimplemented!("Need handle error");
                break;
            } else {
                current_level = next_level;
            }
        }

        self.running_tasks.remove(&task.uuid);

        match self.tasks.get_mut(&task.uuid) {
            Some(mut ref_mut) => {
                let task = ref_mut.value_mut();
                if shutdown_flag {
                    task.state = BackupState::Suspended;
                } else {
                    task.state = BackupState::Completed;
                }
            }
            None => log!(TaskError::TaskNotFound),
        }
    }

    fn to_backup_worker(&self) -> BackupWorker {
        let config = self.config.clone();
        let io_manager = self.io_manager.clone();
        let progress_tracker = self.progress_tracker.clone();
        BackupWorker::new(config, io_manager, progress_tracker)
    }
}

struct BackupWorker {
    config: Arc<AppConfig>,
    io_manager: Arc<IOManager>,
    progress_tracker: Arc<ProgressTracker>,
}

impl BackupWorker {
    pub fn new(
        config: Arc<AppConfig>,
        io_manager: Arc<IOManager>,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Self {
        Self {
            config,
            io_manager,
            progress_tracker,
        }
    }

    async fn run(
        &self,
        task: BackupTask,
        global_queue: Arc<SegQueue<PathBuf>>,
        mut shutdown: oneshot::Receiver<()>,
    ) -> (Vec<PathBuf>, Vec<Error>) {
        let io_manager = &self.io_manager;

        let mirror = task.options.mirror;

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
                match self.process_entry(&task, entry).await {
                    Ok(Some(path)) => next_level.push(path),
                    Ok(None) => {}
                    Err(e) => errors.push(e),
                }
            }

            if mirror {
                let source_entries = entries;
                let destination_dir = match self.calculate_destination_path(
                    &current_dir,
                    &task.source_path,
                    &task.destination_path,
                ) {
                    Ok(dir) => dir,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };
                match io_manager.list_directory(&destination_dir).await {
                    Ok(destination_entries) => {
                        let (_, mirror_errors) = self
                            .mirror_cleanup(source_entries, destination_entries)
                            .await;
                        errors.extend(mirror_errors);
                    }
                    Err(e) => errors.push(e),
                }
            }
        }

        (next_level, errors)
    }

    async fn process_entry(
        &self,
        task: &BackupTask,
        current_path: &PathBuf,
    ) -> Result<Option<PathBuf>, Error> {
        let io_manager = &self.io_manager;

        let source_root = &task.source_path;
        let destination_root = &task.destination_path;

        let source_path = current_path.clone();
        let destination_path =
            self.calculate_destination_path(&source_path, &source_root, &destination_root)?;

        let is_symlink = io_manager.is_symlink(&source_path).await.unwrap_or(false);

        if is_symlink {
            self.process_symlink(task, &source_path, &destination_path)
                .await?;
            return Ok(None);
        }

        if source_path.is_dir() {
            self.backup_directory(task, &source_path, &destination_path)
                .await
        } else {
            self.backup_file(task, &source_path, &destination_path)
                .await
        }
    }

    async fn backup_directory(
        &self,
        task: &BackupTask,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> Result<Option<PathBuf>, Error> {
        let io_manager = &self.io_manager;

        if !destination_path.exists() {
            io_manager.create_directory(&destination_path).await?;
        }

        io_manager
            .copy_attributes(source_path, destination_path)
            .await?;

        if task.options.backup_permission {
            io_manager
                .copy_permission(source_path, destination_path)
                .await?;
        }

        Ok(Some(source_path.clone()))
    }

    async fn backup_file(
        &self,
        task: &BackupTask,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> Result<Option<PathBuf>, Error> {
        let io_manager = &self.io_manager;

        #[allow(unused_variables)]
        let mut file_lock = None;
        #[allow(unused_assignments)]
        if task.options.lock_source {
            file_lock = Some(io_manager.acquire_file_lock(source_path).await?);
        }

        match task.backup_type {
            BackupType::Full => self.full_backup(source_path, destination_path).await?,
            BackupType::Incremental => {
                let comparison_mode = task.comparison_mode.ok_or(SystemError::UnknownError)?;
                self.incremental_backup(source_path, destination_path, comparison_mode)
                    .await?
            }
        }

        io_manager
            .copy_attributes(source_path, destination_path)
            .await?;

        if task.options.backup_permission {
            io_manager
                .copy_permission(source_path, destination_path)
                .await?;
        }

        drop(file_lock);

        Ok(None)
    }

    #[inline(always)]
    async fn process_symlink(
        &self,
        task: &BackupTask,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> Result<(), Error> {
        if task.options.follow_symlinks {
            self.follow_symlink(task, source_path, destination_path)
                .await
        } else {
            self.copy_symlink(task, source_path, destination_path).await
        }
    }

    async fn follow_symlink(
        &self,
        task: &BackupTask,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> Result<(), Error> {
        let io_manager = &self.io_manager;

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
                self.backup_directory(task, &canonical_path, &current_dest)
                    .await?;

                let entries = io_manager.list_directory(&canonical_path).await?;
                for entry in entries {
                    let relative_path = match entry.strip_prefix(&canonical_path) {
                        Ok(rel_path) => rel_path.to_path_buf(),
                        Err(_) => match entry.file_name() {
                            Some(name) => PathBuf::from(name),
                            None => continue,
                        },
                    };
                    let new_destination = current_dest.join(relative_path);
                    queue.push_back((entry, new_destination));
                }
            } else {
                self.backup_file(task, &canonical_path, &current_dest)
                    .await?;
            }
        }

        Ok(())
    }

    async fn copy_symlink(
        &self,
        task: &BackupTask,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> Result<(), Error> {
        let io_manager = &self.io_manager;

        io_manager
            .copy_symlink(source_path, destination_path)
            .await?;

        io_manager
            .copy_attributes(source_path, destination_path)
            .await?;

        if task.options.backup_permission {
            io_manager
                .copy_permission(source_path, destination_path)
                .await?;
        }

        Ok(())
    }

    #[inline(always)]
    async fn full_backup(
        &self,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> Result<(), Error> {
        let io_manager = &self.io_manager;
        io_manager.copy_file(source_path, destination_path).await
    }

    async fn incremental_backup(
        &self,
        source_path: &PathBuf,
        destination_path: &PathBuf,
        comparison_mode: ComparisonMode,
    ) -> Result<(), Error> {
        let io_manager = &self.io_manager;

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
        &self,
        source_entries: Vec<PathBuf>,
        destination_entries: Vec<PathBuf>,
    ) -> ((), Vec<Error>) {
        let io_manager = &self.io_manager;

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
        &self,
        source_path: &PathBuf,
        source_root: &PathBuf,
        destination_root: &PathBuf,
    ) -> Result<PathBuf, Error> {
        let relative_path = source_path
            .strip_prefix(source_root)
            .map_err(|_| SystemError::UnknownError)?;
        Ok(destination_root.join(relative_path))
    }
}

#[async_trait]
pub trait RunBackupEngine {
    async fn run(&self) -> oneshot::Sender<()>;
}

#[async_trait]
impl RunBackupEngine for Arc<BackupEngine> {
    async fn run(&self) -> oneshot::Sender<()> {
        let backup_engine = self.clone();
        let event_bus = self.event_bus.clone();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

        let _ = tokio::spawn(async move {
            let add_task = event_bus.subscribe::<TaskAddRequested>();
            let remove_task = event_bus.subscribe::<TaskRemoveRequested>();
            let start_task = event_bus.subscribe::<TaskStartRequested>();
            let resume_task = event_bus.subscribe::<TaskRemoveRequested>();
            let suspend_task = event_bus.subscribe::<TaskStartRequested>();
            loop {
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }

                while let Ok(event) = add_task.try_recv() {
                    backup_engine.add_task(event.task).await;
                }

                while let Ok(event) = remove_task.try_recv() {
                    backup_engine.remove_task(&event.task_id).await;
                }

                while let Ok(event) = start_task.try_recv() {
                    //todo Need add error handle
                    let _ = backup_engine.start_task(event.task_id).await;
                }

                while let Ok(event) = resume_task.try_recv() {
                    //todo Need add error handle
                    let _ = backup_engine.resume_task(event.task_id).await;
                }

                while let Ok(event) = suspend_task.try_recv() {
                    //todo Need add error handle
                    let _ = backup_engine.suspend_task(event.task_id).await;
                }
            }
        });

        shutdown_tx
    }
}
