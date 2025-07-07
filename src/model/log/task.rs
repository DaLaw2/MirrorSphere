use crate::loggable;

loggable! {
    TaskLog {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
