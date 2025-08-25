use crate::model::core::backup::execution::Execution;

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
    pub execution: Execution,
    pub current_folder: String,
    pub processed_files: usize,
    pub error_count: usize,
}

impl From<Execution> for ExecutionDisplay {
    fn from(execution: Execution) -> Self {
        Self {
            execution,
            current_folder: String::new(),
            processed_files: 0,
            error_count: 0,
        }
    }
}
