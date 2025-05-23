use crate::r#macro::loggable::loggable;

loggable! {
    TaskLog {
        #[error("Placeholder")]
        Placeholder: tracing::Level::INFO,
    }
}
