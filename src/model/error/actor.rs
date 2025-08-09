use macros::traceable;

traceable! {
    ActorError {
        #[no_source]
        #[error("Actor not found")]
        ActorNotFound => tracing::Level::ERROR,
        #[no_source]
        #[error("Actor not responding")]
        ActorNotResponding => tracing::Level::WARN,
        #[no_source]
        #[error("Failed to send message to actor")]
        SendMessageError => tracing::Level::ERROR,
    }
}
