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
    }
}
