use crate::define_log_entries;

define_log_entries! {
    TaskEntry {
        #[error("Task not found")]
        TaskNotFound: tracing::Level::ERROR,
    }
}
