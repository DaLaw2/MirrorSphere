use macros::loggable;

loggable! {
    MiscLog {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
