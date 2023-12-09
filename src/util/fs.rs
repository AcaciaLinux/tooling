//! Utilities for interacting with a filesystem

mod walk;
pub use walk::*;

mod unwind_symlinks;
pub use unwind_symlinks::*;

mod fsentry;
pub use fsentry::*;

use crate::error::{Error, ErrorExt};
use log::trace;
use std::path::Path;

/// Creates a directory
///
/// Uses the [std::fs::create_dir()] function
pub fn create_dir(path: &Path) -> Result<(), Error> {
    trace!("Creating directory '{}'", path.to_string_lossy());
    std::fs::create_dir(path)
        .e_context(|| format!("Creating directory '{}'", path.to_string_lossy()))
}

/// Creates a directory and all of its parents
///
/// Uses the [std::fs::create_dir_all()] function
pub fn create_dir_all(path: &Path) -> Result<(), Error> {
    trace!("Creating directory '{}'", path.to_string_lossy());
    std::fs::create_dir_all(path)
        .e_context(|| format!("Creating directory '{}'", path.to_string_lossy()))
}

/// Creates a symlink pointing to `destination`
///
/// Uses the [std::os::unix::fs::symlink()] function
pub fn create_symlink(path: &Path, destination: &Path) -> Result<(), Error> {
    trace!(
        "Creating symlink '{}' pointing to '{}'",
        path.to_string_lossy(),
        destination.to_string_lossy()
    );
    std::os::unix::fs::symlink(destination, path).e_context(|| {
        format!(
            "Creating symlink '{}' pointing to '{}'",
            path.to_string_lossy(),
            destination.to_string_lossy()
        )
    })
}
