use crate::r#macro::loggable::loggable;

loggable! {
    DatabaseLog {
        #[error("Connected to database successfully")]
        DatabaseConnectSuccess => tracing::Level::INFO,
    }
}
