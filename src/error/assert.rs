//! Assertion errors
use std::path::PathBuf;

use crate::GIT_COMMIT_HASH;

/// An error for an assertion, includes the path to where the assertion has been made
#[derive(Debug)]
pub struct AssertionError {
    /// The error
    pub error: AssertionErrorType,
    /// The line number where the assertion has been made
    pub line: u32,
    /// The file where the assertion has been made
    pub file: String,
}

/// All the possible assertion errors
#[derive(Debug)]
pub enum AssertionErrorType {
    /// A path was expected to be relative
    RelativePath(PathBuf),
    /// A path was expected to be absolute
    AbsolutePath(PathBuf),
}

impl std::fmt::Display for AssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match &self.error {
            AssertionErrorType::RelativePath(path) => {
                format!("Expected '{}' to be relative", path.to_string_lossy())
            }
            AssertionErrorType::AbsolutePath(path) => {
                format!("Expected '{}' to be absolute", path.to_string_lossy())
            }
        };

        let msg = format!(
            "[GIT: {}] {}:{} - {}",
            GIT_COMMIT_HASH, self.file, self.line, msg
        );
        let len = msg.len();

        writeln!(f, "{}", msg)?;

        writeln!(f, "!!!!{}!!!!", "!".repeat(len))?;
        writeln!(
            f,
            "!!  {:^width$}  !!",
            "ASSERTION ERROR - THIS IS A BUG!",
            width = len
        )?;
        writeln!(
            f,
            "!!  {:^width$}  !!",
            "Please report this error to",
            width = len
        )?;
        writeln!(
            f,
            "!!  {:^width$}  !!",
            "https://github.com/AcaciaLinux/tooling/issues",
            width = len
        )?;
        writeln!(f, "!!  {:^width$}  !!", msg, width = len)?;
        write!(f, "!!!!{}!!!!", "!".repeat(len))
    }
}
