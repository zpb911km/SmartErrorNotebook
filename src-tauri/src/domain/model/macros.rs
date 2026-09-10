macro_rules! pub_string_enum {
    (
        $name:ident {
            $($variant:ident => $value:literal),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            $($variant,)+
        }

        impl $name {
            pub fn from_str(value: &str) -> Result<Self, &'static str> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err("invalid value"),
                }
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $value,)+
                }
            }
        }

        impl std::str::FromStr for $name {
            type Err = &'static str;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::from_str(value)
            }
        }

        impl From<$name> for &str {
            fn from(value: $name) -> Self {
                value.as_str()
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.as_str().to_owned()
            }
        }
    };
}

pub(super) use pub_string_enum;
