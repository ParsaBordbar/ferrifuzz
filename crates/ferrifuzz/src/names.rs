//! `named_enum!` — string names for option enums (`"min"`, `"centroid"`, ...).

/// Gives a fieldless enum `ALL`, `name()`, `Display` and a case-insensitive `FromStr`
/// whose error lists the valid names and suggests the closest one.
macro_rules! named_enum {
    ($ty:ident, $what:literal, { $($variant:ident => $canon:literal $(| $alias:literal)*),+ $(,)? }) => {
        impl $ty {
            /// Every variant, in declaration order.
            pub const ALL: &'static [$ty] = &[$($ty::$variant),+];

            /// Canonical name — what `Display` prints and `FromStr` accepts.
            pub fn name(self) -> &'static str {
                match self {
                    $($ty::$variant => $canon),+
                }
            }
        }

        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.name())
            }
        }

        impl std::str::FromStr for $ty {
            type Err = crate::Error;

            fn from_str(s: &str) -> crate::Result<Self> {
                Ok(match s.trim().to_lowercase().as_str() {
                    $($canon $(| $alias)* => $ty::$variant,)+
                    _ => return Err(crate::error::unknown_name($what, s, &[$($canon),+])),
                })
            }
        }
    };
}
