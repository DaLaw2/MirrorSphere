use crate::r#macro::loggable::loggable;

loggable! {
    EventError {
        #[error("Placeholder")]
        Placeholder: tracing::Level::INFO,
    }
}
