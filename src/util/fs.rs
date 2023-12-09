//! Utilities for interacting with a filesystem

mod walk;
pub use walk::*;

mod unwind_symlinks;
pub use unwind_symlinks::*;

mod fsentry;
pub use fsentry::*;

use crate::error::{Error, ErrorExt};
use log::trace;
use std::{fs::File, path::Path};

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

/// Opens a file using the [std::fs::File::open()] function
/// # Arguments
/// * `path` - The path to the file to open
pub fn file_open(path: &Path) -> Result<File, Error> {
    File::open(path).e_context(|| format!("Opening file {}", path.to_string_lossy()))
}

/// Creates a file using the [std::fs::File::create()] function
/// # Arguments
/// * `path` - The path to the file to create
pub fn file_create(path: &Path) -> Result<File, Error> {
    File::create(path).e_context(|| format!("Creating file {}", path.to_string_lossy()))
}
