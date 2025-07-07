use crate::loggable;

loggable! {
    IOLog {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
