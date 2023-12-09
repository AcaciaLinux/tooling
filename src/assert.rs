//! Assertion functions

use std::path::Path;

use crate::error::{
    assert::{AssertionError, AssertionErrorType},
    Error, Throwable,
};

/// Asserts that a path is relative
/// # Arguments
/// * `path` - The path to check
/// # Returns
/// The path or an assertion error
#[macro_export]
macro_rules! assert_relative {
    ($path: expr) => {
        $crate::assert::assert_relative_raw($path, || (line!(), file!()))
    };
}

/// Asserts that a path is relative
///
/// Consider using the `assert_relative!()` macro
/// # Arguments
/// * `path` - The path to check
/// * `callback` - The callback to provide the following tuple: (line, file)
/// # Returns
/// The path or an assertion error
pub fn assert_relative_raw<'a, F: Fn() -> (u32, &'a str)>(
    path: &'a Path,
    callback: F,
) -> Result<&'a Path, Error> {
    if path.is_relative() {
        Ok(path)
    } else {
        let info = callback();
        let error = AssertionError {
            error: AssertionErrorType::RelativePath(path.to_owned()),
            line: info.0,
            file: info.1.to_string(),
        };
        Err(error.throw(format!(
            "Asserting '{}' is relative",
            path.to_string_lossy()
        )))
    }
}

/// Asserts that a path is absolute
/// # Arguments
/// * `path` - The path to check
/// # Returns
/// The path or an assertion error
#[macro_export]
macro_rules! assert_absolute {
    ($path: expr) => {
        $crate::assert::assert_absolute_raw($path, || (line!(), file!()))
    };
}

/// Asserts that a path is absolute
///
/// Consider using the `assert_absolute!()` macro
/// # Arguments
/// * `path` - The path to check
/// * `callback` - The callback to provide the following tuple: (line, file)
/// # Returns
/// The path or an assertion error
pub fn assert_absolute_raw<'a, F: Fn() -> (u32, &'a str)>(
    path: &'a Path,
    callback: F,
) -> Result<&'a Path, Error> {
    if path.is_absolute() {
        Ok(path)
    } else {
        let info = callback();
        let error = AssertionError {
            error: AssertionErrorType::AbsolutePath(path.to_owned()),
            line: info.0,
            file: info.1.to_string(),
        };
        Err(error.throw(format!(
            "Asserting '{}' is absolute",
            path.to_string_lossy()
        )))
    }
}
