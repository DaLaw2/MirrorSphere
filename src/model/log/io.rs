use crate::r#macro::loggable::loggable;

loggable! {
    IOLog {
        #[error("Placeholder")]
        Placeholder: tracing::Level::INFO,
    }
}
