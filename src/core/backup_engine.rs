use crate::core::app_config::AppConfig;
use crate::core::event_bus::EventBus;
use crate::core::io_manager::IOManager;
use crate::core::progress_tracker::ProgressTracker;
use crate::interface::file_system::FileSystemTrait;
use crate::interface::service_unit::ServiceUnit;
use crate::model::backup::backup_execution::*;
use crate::model::error::Error;
use crate::model::error::system::SystemError;
use crate::model::error::task::TaskError;
use crate::model::event::execution::*;
use async_trait::async_trait;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use futures::future::join_all;
use macros::log;
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::error;
use uuid::Uuid;

pub struct BackupEngine {
    app_config: Arc<AppConfig>,
    event_bus: Arc<EventBus>,
    io_manager: Arc<IOManager>,
    progress_tracker: Arc<ProgressTracker>,
    executions: Arc<DashMap<Uuid, BackupExecution>>,
    running_executions: Arc<DashMap<Uuid, (oneshot::Sender<()>, JoinHandle<()>)>>,
}

impl BackupEngine {
    pub fn new(
        app_config: Arc<AppConfig>,
        event_bus: Arc<EventBus>,
        io_manager: Arc<IOManager>,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Self {
        Self {
            app_config,
            event_bus,
            io_manager,
            progress_tracker,
            executions: Arc::new(DashMap::new()),
            running_executions: Arc::new(DashMap::new()),
        }
    }

    pub async fn stop_all_executions(&self) {
        let keys: Vec<Uuid> = self
            .running_executions
            .iter()
            .map(|pair| pair.key().clone())
            .collect();
        for uuid in keys {
            if let Some((_, (shutdown, handle))) = self.running_executions.remove(&uuid) {
                if shutdown.send(()).is_err() {
                    log!(SystemError::ShutdownSignalFailed);
                    continue;
                }
                if let Err(err) = handle.await {
                    log!(SystemError::ThreadPanic(err));
                }
            }
        }
    }

    pub async fn add_execution(&self, execution: BackupExecution) {
        self.executions.insert(execution.uuid, execution);
    }

    pub async fn remove_execution(&self, uuid: &Uuid) {
        self.executions.remove(uuid);
    }

    pub async fn start_execution(&self, uuid: Uuid) -> Result<(), Error> {
        if self.running_executions.contains_key(&uuid) {
            Err(TaskError::IllegalRunState)?
        }

        let mut ref_mut = self
            .executions
            .get_mut(&uuid)
            .ok_or(TaskError::ExecutionNotFound)?;
        let execution = ref_mut.value_mut();
        if execution.state != BackupState::Pending {
            Err(TaskError::IllegalRunState)?
        }
        execution.state = BackupState::Running;

        let execution_runner = self.to_execution_runner();
        let execution = execution.clone();
        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { execution_runner.run(execution, rx, false).await });
        self.running_executions.insert(uuid, (tx, handle));
        Ok(())
    }

    pub async fn suspend_execution(&self, uuid: Uuid) -> Result<(), Error> {
        let mut ref_mut = self
            .executions
            .get_mut(&uuid)
            .ok_or(TaskError::ExecutionNotFound)?;
        let execution = ref_mut.value_mut();
        if execution.state != BackupState::Running {
            Err(TaskError::IllegalRunState)?
        }
        execution.state = BackupState::Suspended;
        drop(ref_mut);

        let (_, (shutdown, handle)) = self
            .running_executions
            .remove(&uuid)
            .ok_or(TaskError::ExecutionNotFound)?;
        shutdown
            .send(())
            .map_err(|_| SystemError::ShutdownSignalFailed)?;
        handle.await.map_err(SystemError::ThreadPanic)?;
        Ok(())
    }

    pub async fn resume_execution(&self, uuid: Uuid) -> Result<(), Error> {
        if self.running_executions.contains_key(&uuid) {
            Err(TaskError::IllegalRunState)?
        }

        let mut ref_mut = self
            .executions
            .get_mut(&uuid)
            .ok_or(TaskError::ExecutionNotFound)?;
        let execution = ref_mut.value_mut();
        if execution.state != BackupState::Suspended {
            Err(TaskError::IllegalRunState)?
        }
        execution.state = BackupState::Running;

        let execution_runner = self.to_execution_runner();
        let execution = execution.clone();
        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { execution_runner.run(execution, rx, true).await });
        self.running_executions.insert(uuid, (tx, handle));
        Ok(())
    }

    fn to_execution_runner(&self) -> ExecutionRunner {
        let config = self.app_config.clone();
        let io_manager = self.io_manager.clone();
        let progress_tracker = self.progress_tracker.clone();
        let executions = self.executions.clone();
        let running_executions = self.running_executions.clone();
        ExecutionRunner::new(
            config,
            io_manager,
            progress_tracker,
            executions,
            running_executions,
        )
    }
}

struct ExecutionRunner {
    app_config: Arc<AppConfig>,
    io_manager: Arc<IOManager>,
    progress_tracker: Arc<ProgressTracker>,
    executions: Arc<DashMap<Uuid, BackupExecution>>,
    running_executions: Arc<DashMap<Uuid, (oneshot::Sender<()>, JoinHandle<()>)>>,
}

impl ExecutionRunner {
    pub fn new(
        app_config: Arc<AppConfig>,
        io_manager: Arc<IOManager>,
        progress_tracker: Arc<ProgressTracker>,
        executions: Arc<DashMap<Uuid, BackupExecution>>,
        running_executions: Arc<DashMap<Uuid, (oneshot::Sender<()>, JoinHandle<()>)>>,
    ) -> Self {
        Self {
            app_config,
            io_manager,
            progress_tracker,
            executions,
            running_executions,
        }
    }

    async fn run(
        &self,
        execution: BackupExecution,
        mut shutdown: oneshot::Receiver<()>,
        resume: bool,
    ) {
        let config = &self.app_config;
        let progress_tracker = &self.progress_tracker;

        let (mut current_level, mut errors) = if resume {
            progress_tracker.resume_execution(execution.uuid).await
        } else {
            let source_root = execution.source_path.clone();
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
                let worker = self.to_worker();
                let (tx, rx) = oneshot::channel();
                let execution = execution.clone();
                let queue = global_queue.clone();
                let handle = tokio::spawn(async move { worker.run(execution, queue, rx).await });
                worker_shutdowns.push(tx);
                worker_handles.push(handle);
            }

            let workers_results = tokio::select! {
                results = join_all(&mut worker_handles) => results,
                _ = &mut shutdown => {
                    shutdown_flag = true;
                    for shutdown in worker_shutdowns {
                        if shutdown.send(()).is_err() {
                            log!(SystemError::ShutdownSignalFailed);
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
                    Err(err) => log!(SystemError::ThreadPanic(err)),
                }
            }

            if shutdown_flag {
                current_level.extend(next_level);
                if let Err(err) = progress_tracker
                    .save_execution(execution.uuid, current_level, errors)
                    .await {
                    error!("{}", err);
                }
                break;
            } else {
                current_level = next_level;
            }
        }

        self.running_executions.remove(&execution.uuid);

        match self.executions.get_mut(&execution.uuid) {
            Some(mut ref_mut) => {
                let execution = ref_mut.value_mut();
                if shutdown_flag {
                    execution.state = BackupState::Suspended;
                } else {
                    execution.state = BackupState::Completed;
                }
            }
            None => log!(TaskError::ExecutionNotFound),
        }
    }

    fn to_worker(&self) -> Worker {
        let io_manager = self.io_manager.clone();
        Worker::new(io_manager)
    }
}

struct Worker {
    io_manager: Arc<IOManager>,
}

impl Worker {
    pub fn new(io_manager: Arc<IOManager>) -> Self {
        Self {
            io_manager
        }
    }

    async fn run(
        &self,
        execution: BackupExecution,
        global_queue: Arc<SegQueue<PathBuf>>,
        mut shutdown: oneshot::Receiver<()>,
    ) -> (Vec<PathBuf>, Vec<Error>) {
        let io_manager = &self.io_manager;

        let mirror = execution.options.mirror;

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
                match self.process_entry(&execution, entry).await {
                    Ok(Some(path)) => next_level.push(path),
                    Ok(None) => {}
                    Err(e) => errors.push(e),
                }
            }

            if mirror {
                let source_entries = entries;
                let destination_dir = match self.calculate_destination_path(
                    &current_dir,
                    &execution.source_path,
                    &execution.destination_path,
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
        execution: &BackupExecution,
        current_path: &PathBuf,
    ) -> Result<Option<PathBuf>, Error> {
        let io_manager = &self.io_manager;

        let source_root = &execution.source_path;
        let destination_root = &execution.destination_path;

        let source_path = current_path.clone();
        let destination_path =
            self.calculate_destination_path(&source_path, &source_root, &destination_root)?;

        let is_symlink = io_manager.is_symlink(&source_path).await.unwrap_or(false);

        if is_symlink {
            self.process_symlink(execution, &source_path, &destination_path)
                .await?;
            return Ok(None);
        }

        if source_path.is_dir() {
            self.backup_directory(execution, &source_path, &destination_path)
                .await
        } else {
            self.backup_file(execution, &source_path, &destination_path)
                .await
        }
    }

    async fn backup_directory(
        &self,
        execution: &BackupExecution,
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

        if execution.options.backup_permission {
            io_manager
                .copy_permission(source_path, destination_path)
                .await?;
        }

        Ok(Some(source_path.clone()))
    }

    async fn backup_file(
        &self,
        execution: &BackupExecution,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> Result<Option<PathBuf>, Error> {
        let io_manager = &self.io_manager;

        #[allow(unused_variables)]
        let mut file_lock = None;
        #[allow(unused_assignments)]
        if execution.options.lock_source {
            file_lock = Some(io_manager.acquire_file_lock(source_path).await?);
        }

        match execution.backup_type {
            BackupType::Full => self.full_backup(source_path, destination_path).await?,
            BackupType::Incremental => {
                let comparison_mode = execution.comparison_mode.ok_or(SystemError::UnknownError)?;
                self.incremental_backup(source_path, destination_path, comparison_mode)
                    .await?
            }
        }

        io_manager
            .copy_attributes(source_path, destination_path)
            .await?;

        if execution.options.backup_permission {
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
        execution: &BackupExecution,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> Result<(), Error> {
        if execution.options.follow_symlinks {
            self.follow_symlink(execution, source_path, destination_path)
                .await
        } else {
            self.copy_symlink(execution, source_path, destination_path)
                .await
        }
    }

    async fn follow_symlink(
        &self,
        execution: &BackupExecution,
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
                self.backup_directory(execution, &canonical_path, &current_dest)
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
                self.backup_file(execution, &canonical_path, &current_dest)
                    .await?;
            }
        }

        Ok(())
    }

    async fn copy_symlink(
        &self,
        execution: &BackupExecution,
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

        if execution.options.backup_permission {
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
                    } else if let Err(e) = io_manager.delete_file(&dest_entry).await {
                        errors.push(e);
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
            .map_err(SystemError::UnexpectError)?;
        Ok(destination_root.join(relative_path))
    }
}

#[async_trait]
impl ServiceUnit for BackupEngine {
    async fn run_impl(self: Arc<Self>, mut shutdown_rx: oneshot::Receiver<()>) {
        let backup_engine = self.clone();
        let event_bus = self.event_bus.clone();
        let add_execution = event_bus.subscribe::<ExecutionAddRequest>();
        let remove_execution = event_bus.subscribe::<ExecutionRemoveRequest>();
        let start_execution = event_bus.subscribe::<ExecutionStartRequest>();
        let resume_execution = event_bus.subscribe::<ExecutionResumeRequested>();
        let suspend_execution = event_bus.subscribe::<ExecutionSuspendRequest>();
        loop {
            if shutdown_rx.try_recv().is_ok() {
                break;
            }

            while let Ok(event) = add_execution.try_recv() {
                backup_engine.add_execution(event.execution).await;
            }
            while let Ok(event) = remove_execution.try_recv() {
                backup_engine.remove_execution(&event.execution_id).await;
            }
            while let Ok(event) = start_execution.try_recv() {
                if let Err(err) = backup_engine.start_execution(event.execution_id).await {
                    error!("{}", err);
                }
            }
            while let Ok(event) = resume_execution.try_recv() {
                if let Err(err) = backup_engine.resume_execution(event.execution_id).await {
                    error!("{}", err);
                }
            }
            while let Ok(event) = suspend_execution.try_recv() {
                if let Err(err) = backup_engine.suspend_execution(event.execution_id).await {
                    error!("{}", err);
                }
            }
        }
    }
}
