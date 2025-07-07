#[macro_export]
macro_rules! log {
    ($error:expr) => {
        {
            let error = $error;
            let level = error.level();
            let message = error.to_string();

            match level {
                tracing::Level::ERROR => tracing::error!("{}", message),
                tracing::Level::WARN => tracing::warn!("{}", message),
                tracing::Level::INFO => tracing::info!("{}", message),
                tracing::Level::DEBUG => tracing::debug!("{}", message),
                tracing::Level::TRACE => tracing::trace!("{}", message),
            }
        }
    };
    ($error:expr, $debug_info:expr) => {
        {
            let error = $error;
            let level = error.level();
            let message = error.to_string();
            let debug_info = $debug_info;

            match level {
                tracing::Level::ERROR => tracing::error!(message = %message, debug = ?debug_info),
                tracing::Level::WARN => tracing::warn!(message = %message, debug = ?debug_info),
                tracing::Level::INFO => tracing::info!(message = %message, debug = ?debug_info),
                tracing::Level::DEBUG => tracing::debug!(message = %message, debug = ?debug_info),
                tracing::Level::TRACE => tracing::trace!(message = %message, debug = ?debug_info),
            }
        }
    };
}
