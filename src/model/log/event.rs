use crate::loggable;

loggable! {
    EventLog {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
