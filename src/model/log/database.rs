use crate::loggable;

loggable! {
    DatabaseLog {
        #[error("Connected to database successfully")]
        DatabaseConnectSuccess => tracing::Level::INFO,
    }
}
