#[macro_export]
macro_rules! loggable {
    (
        $enum_name:ident {
            $(
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
        }
    };
}
