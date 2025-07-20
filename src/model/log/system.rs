use macros::loggable;

loggable! {
    SystemLog {
        #[error("Rerun the program as administrator")]
        ReRunAsAdmin => tracing::Level::WARN,

        #[error("Initializing")]
        Initializing => tracing::Level::INFO,

        #[error("Initialization completed")]
        InitializeComplete => tracing::Level::INFO,

        #[error("Termination in process")]
        Terminating => tracing::Level::INFO,

        #[error("Termination completed")]
        TerminateComplete => tracing::Level::INFO,

        #[error("Gui Exited")]
        GuiExited => tracing::Level::INFO,
    }
}
