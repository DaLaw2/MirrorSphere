use macros::traceable;

traceable! {
    MiscError {
        #[error("Failed to free object")]
        ObjectFreeFailed => tracing::Level::ERROR,

        #[error("Failed to serialize object")]
        SerializeError => tracing::Level::ERROR,

        #[error("Failed to deserialize object")]
        DeserializeError => tracing::Level::ERROR,

        #[error("Failed to initialize UI platform")]
        UIPlatformError => tracing::Level::ERROR,

        #[no_source]
        #[error("Assert file not found")]
        AssertFileNotFound => tracing::Level::ERROR,

        #[no_source]
        #[error("Handler not found")]
        HandlerNotFound => tracing::Level::ERROR,

        #[no_source]
        #[error("Type mismatch")]
        TypeMismatch => tracing::Level::ERROR,

        #[no_source]
        #[error("Type not registered")]
        TypeNotRegistered => tracing::Level::ERROR,

        #[no_source]
        #[error("Channel closed")]
        ChannelClosed => tracing::Level::ERROR,

        #[no_source]
        #[error("Channel empty")]
        ChannelEmpty => tracing::Level::INFO,
    }
}
