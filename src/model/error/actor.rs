use macros::traceable;

traceable! {
    ActorError {
        #[no_source]
        #[error("Actor not found")]
        ActorNotFound => tracing::Level::ERROR,
        #[error("Actor not responding")]
        ActorNotResponding => tracing::Level::WARN,
        #[error("Failed to send message to actor")]
        SendMessageError => tracing::Level::ERROR,
    }
}
