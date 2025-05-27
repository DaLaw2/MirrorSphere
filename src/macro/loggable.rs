pub use crate::loggable;

#[macro_export]
macro_rules! loggable {
    (
        $enum_name:ident {
            $(
                $(#[doc = $doc:expr])*
                #[error($msg:expr)]
                $variant:ident $({ $($field:ident: $field_type:ty),* $(,)? })? => $level:expr
                $(,)?
            )*
        }
    ) => {
        #[allow(dead_code)]
        #[derive(Debug, Clone, thiserror::Error, serde::Serialize, serde::Deserialize)]
        pub enum $enum_name {
            $(
                $(#[doc = $doc])*
                #[error($msg)]
                $variant $({ $($field: $field_type),* })?,
            )*
        }

        impl $enum_name {
            #[allow(dead_code)]
            pub fn level(&self) -> tracing::Level {
                match self {
                    $(Self::$variant $({ $($field: _),* })? => $level,)*
                }
            }

            #[allow(dead_code)]
            pub fn log(&self) {
                let level = self.level();
                let message = self.to_string();

                match level {
                    tracing::Level::ERROR => tracing::error!("{}", message),
                    tracing::Level::WARN => tracing::warn!("{}", message),
                    tracing::Level::INFO => tracing::info!("{}", message),
                    tracing::Level::DEBUG => tracing::debug!("{}", message),
                    tracing::Level::TRACE => tracing::trace!("{}", message),
                }
            }

            #[allow(dead_code)]
            pub fn log_with_context<T: std::fmt::Display>(&self, context: T) {
                let level = self.level();
                let message = self.to_string();
                let context = context.to_string();

                match level {
                    tracing::Level::ERROR => tracing::error!(message = %message, context = %context),
                    tracing::Level::WARN => tracing::warn!(message = %message, context = %context),
                    tracing::Level::INFO => tracing::info!(message = %message, context = %context),
                    tracing::Level::DEBUG => tracing::debug!(message = %message, context = %context),
                    tracing::Level::TRACE => tracing::trace!(message = %message, context = %context),
                }
            }
        }
    };
}
