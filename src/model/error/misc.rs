use crate::r#macro::loggable::loggable;

loggable! {
    MiscError {
        #[error("Failed to free object")]
        ObjectFreeFailed: tracing::Level::ERROR,
    }
}
