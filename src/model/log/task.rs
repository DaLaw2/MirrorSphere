use macros::loggable;

loggable! {
    TaskLog {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
