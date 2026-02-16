//! Filesystem operation utilities wrapped in thin
//! error-abstracting wrappers

use crate::str;
use std::{fs::File, path::Path};

use log::trace;

use super::PathExt;
use crate::error::{ALErrorExt, ALResult};

/// Creates a directory
///
/// Uses the [std::fs::create_dir()] function
pub fn mkdir(path: &Path) -> ALResult<()> {
    trace!("Creating directory '{}'", path.to_string_lossy());
    std::fs::create_dir(path).ctx(str!("Creating directory '{}'", path.to_string_lossy()))
}

/// Creates a directory and all of its parents
///
/// Uses the [std::fs::create_dir_all()] function
pub fn mkdir_p(path: &Path) -> ALResult<()> {
    trace!("Creating directory '{}'", path.to_string_lossy());
    std::fs::create_dir_all(path).ctx(str!("Creating directory '{}'", path.to_string_lossy()))
}

/// Copies `src` to `dest`
///
/// Uses the [std::fs::copy()] function
pub fn file_copy(src: &Path, dest: &Path) -> ALResult<u64> {
    trace!("Copying {} ==> {}", src.str_lossy(), dest.str_lossy());
    std::fs::copy(src, dest).ctx(str!(
        "Copying '{}' to '{}'",
        src.to_string_lossy(),
        dest.to_string_lossy()
    ))
}

/// Renames `src` to `dest`
///
/// Uses the [std::fs::rename()] function
pub fn rename(src: &Path, dest: &Path) -> ALResult<()> {
    trace!("Renaming {} ==> {}", src.str_lossy(), dest.str_lossy());
    std::fs::rename(src, dest).ctx(str!(
        "Renaming '{}' to '{}'",
        src.to_string_lossy(),
        dest.to_string_lossy()
    ))
}

/// Remove a file
///
/// Uses the [std::fs::remove_file()] function
pub fn file_rm(path: &Path) -> ALResult<()> {
    trace!("Removing file {}", path.str_lossy());
    std::fs::remove_file(path).ctx(str!("Removing file '{}'", path.to_string_lossy()))
}

/// Remove an empty directory
///
/// Uses the [std::fs::remove_dir()] function
pub fn dir_rm(path: &Path) -> ALResult<()> {
    trace!("Removing directory {}", path.str_lossy());
    std::fs::remove_dir(path).ctx(str!(
        "Removing empty directory '{}'",
        path.to_string_lossy()
    ))
}

/// Remove a directory and all of its contents
///
/// Uses the [std::fs::remove_dir_all()] function
pub fn dir_rm_r(path: &Path) -> ALResult<()> {
    trace!("Removing directory recursively {}", path.str_lossy());
    std::fs::remove_dir_all(path).ctx(str!(
        "Removing empty directory '{}'",
        path.to_string_lossy()
    ))
}

/// Opens a file using the [std::fs::File::open()] function
/// # Arguments
/// * `path` - The path to the file to open
pub fn file_open(path: &Path) -> ALResult<File> {
    File::open(path).ctx(str!("Opening file {}", path.to_string_lossy()))
}

/// Creates a file using the [std::fs::File::create()] function
/// # Arguments
/// * `path` - The path to the file to create
pub fn file_create(path: &Path) -> ALResult<File> {
    trace!("Creating file {}", path.str_lossy());
    File::create(path).ctx(str!("Creating file {}", path.to_string_lossy()))
}

/// Creates and opens a file in read and write mode.
/// # Arguments
/// * `path` - The path to the file to create
pub fn file_create_rw(path: &Path) -> ALResult<File> {
    trace!("Creating file RW {}", path.str_lossy());
    File::options()
        .create(true)
        .append(false)
        .truncate(true)
        .read(true)
        .write(true)
        .open(path)
        .ctx(str!("Creating file {}", path.to_string_lossy()))
}

/// Reads the contents of `path` to a string
///
/// Uses the [std::fs::read_to_string] function
/// # Arguments
/// * `path` - The path to the file to read
pub fn file_read_to_string(path: &Path) -> ALResult<String> {
    trace!("Reading file {}", path.str_lossy());
    std::fs::read_to_string(path).ctx(str!("Reading {} to string", path.to_string_lossy()))
}
