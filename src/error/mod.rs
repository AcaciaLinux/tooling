//! The error handling framework

mod error_struct;
pub use error_struct::{ALError, ALErrorExt, ALResult};

mod kind;
pub use kind::ALErrorKind;

mod context;
pub use context::ALErrorContext;

/// Utility macro that shortens `|| format!(...)` for use in `.ctx()`
#[macro_export]
macro_rules! str {
    ($($arg:tt)*) => {{
        || format!($($arg)*)
    }};
}
