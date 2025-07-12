use macros::loggable;

loggable! {
    EventLog {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
