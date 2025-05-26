use crate::r#macro::loggable::loggable;

loggable! {
    EventLog {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
