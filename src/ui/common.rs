use crate::model::core::backup::backup_execution::BackupExecution;

#[derive(Debug, Clone, PartialEq)]
pub enum PageType {
    Executions,
    Schedules,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FolderSelectionMode {
    Source,
    Destination,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonModeSelection {
    Standard,
    Advanced,
    Thorough,
}

#[derive(Debug, Clone)]
pub struct ExecutionDisplay {
    pub execution: BackupExecution,
    pub current_folder: String,
    pub processed_files: usize,
    pub error_count: usize,
}

impl From<BackupExecution> for ExecutionDisplay {
    fn from(execution: BackupExecution) -> Self {
        Self {
            execution,
            current_folder: String::new(),
            processed_files: 0,
            error_count: 0,
        }
    }
}
