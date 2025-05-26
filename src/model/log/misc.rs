use crate::r#macro::loggable::loggable;

loggable! {
    MiscLog {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
