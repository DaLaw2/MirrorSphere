use crate::loggable;

loggable! {
    EventError {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
