//! Utilities for interacting with a filesystem

mod unix_info;
pub use unix_info::*;

mod walk;
pub use walk::*;

mod unwind_symlinks;
pub use unwind_symlinks::*;

mod fsentry;
pub use fsentry::*;

mod pathutil;
pub use pathutil::*;

use crate::error::{Error, ErrorExt};
use log::trace;
use std::{
    fs::{self, File},
    path::Path,
};

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

    // If the path exists, try to remove it first
    if path.exists() {
        fs::remove_file(path)
            .e_context(|| format!("Removing existing symlink or file {}", path.str_lossy()))?
    }

    std::os::unix::fs::symlink(destination, path).e_context(|| {
        format!(
            "Creating symlink '{}' pointing to '{}'",
            path.to_string_lossy(),
            destination.to_string_lossy()
        )
    })
}

/// Copies `src` to `dest`
///
/// Uses the [std::fs::copy()] function
pub fn copy(src: &Path, dest: &Path) -> Result<u64, Error> {
    trace!("Copying {} ==> {}", src.str_lossy(), dest.str_lossy());
    std::fs::copy(src, dest).e_context(|| {
        format!(
            "Copying '{}' to '{}'",
            src.to_string_lossy(),
            dest.to_string_lossy()
        )
    })
}

/// Renames `src` to `dest`
///
/// Uses the [std::fs::rename()] function
pub fn rename(src: &Path, dest: &Path) -> Result<(), Error> {
    trace!("Renaming {} ==> {}", src.str_lossy(), dest.str_lossy());
    std::fs::rename(src, dest).e_context(|| {
        format!(
            "Renaming '{}' to '{}'",
            src.to_string_lossy(),
            dest.to_string_lossy()
        )
    })
}

/// Remove a file
///
/// Uses the [std::fs::remove_file()] function
pub fn remove_file(path: &Path) -> Result<(), Error> {
    trace!("Removing file {}", path.str_lossy());
    std::fs::remove_file(path).e_context(|| format!("Removing file '{}'", path.to_string_lossy()))
}

/// Remove an empty directory
///
/// Uses the [std::fs::remove_dir()] function
pub fn remove_dir(path: &Path) -> Result<(), Error> {
    trace!("Removing directory {}", path.str_lossy());
    std::fs::remove_dir(path)
        .e_context(|| format!("Removing empty directory '{}'", path.to_string_lossy()))
}

/// Remove a directory and all of its contents
///
/// Uses the [std::fs::remove_dir_all()] function
pub fn remove_dir_all(path: &Path) -> Result<(), Error> {
    trace!("Removing directory recursively {}", path.str_lossy());
    std::fs::remove_dir_all(path)
        .e_context(|| format!("Removing empty directory '{}'", path.to_string_lossy()))
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
    trace!("Creating file {}", path.str_lossy());
    File::create(path).e_context(|| format!("Creating file {}", path.to_string_lossy()))
}

/// Creates and opens a file in read and write mode.
/// # Arguments
/// * `path` - The path to the file to create
pub fn file_create_rw(path: &Path) -> Result<File, Error> {
    trace!("Creating file RW {}", path.str_lossy());
    File::options()
        .create(true)
        .append(false)
        .truncate(true)
        .read(true)
        .write(true)
        .open(path)
        .e_context(|| format!("Creating file {}", path.to_string_lossy()))
}

/// Reads the contents of `path` to a string
///
/// Uses the [std::fs::read_to_string] function
/// # Arguments
/// * `path` - The path to the file to read
pub fn file_read_to_string(path: &Path) -> Result<String, Error> {
    trace!("Reading file {}", path.str_lossy());
    std::fs::read_to_string(path)
        .e_context(|| format!("Reading {} to string", path.to_string_lossy()))
}
