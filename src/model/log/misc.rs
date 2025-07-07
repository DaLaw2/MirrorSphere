use crate::loggable;

loggable! {
    MiscLog {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
