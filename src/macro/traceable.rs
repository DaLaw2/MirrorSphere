use paste::paste;

#[macro_export]
macro_rules! traceable {
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
        #[derive(Debug, thiserror::Error)]
        pub enum $enum_name {
            $(
                #[error($msg)]
                $variant {
                    $($($field: $field_type,)*)?
                    #[source]
                    source: anyhow::Error
                },
            )*
        }

        impl $enum_name {
            #[allow(dead_code)]
            pub fn level(&self) -> tracing::Level {
                match self {
                    $(Self::$variant { $($($field: _,)*)? source: _ } => $level,)*
                }
            }

            $(
                traceable!(@constructor $variant $($($field: $field_type)*)?);
            )*
        }
    };

    (@constructor $variant:ident $($field:ident: $field_type:ty)*) => {
        paste::paste! {
            #[allow(non_snake_case)]
            pub fn $variant($($field: impl Into<$field_type>,)* source: impl Into<anyhow::Error>) -> Self {
                Self::$variant {
                    $($field: $field.into(),)*
                    source: source.into()
                }
            }
        }
    };
}
