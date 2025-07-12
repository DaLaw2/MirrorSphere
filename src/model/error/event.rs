use macros::traceable;

traceable! {
    EventError {
        #[error("Placeholder")]
        Placeholder => tracing::Level::INFO,
    }
}
