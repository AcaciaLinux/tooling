//! Utilities for handling archives

use super::fs::file_open;
use crate::error::{Error, ErrorExt, Throwable};
use std::{io::Read, path::Path};

/// Tries to determine the archive type and use the according function to extract it
/// # Arguments
/// * `src` - The path to the source archive file
/// * `dest` - The destination directory to extract the archive to
pub fn extract_infer(src: &Path, dest: &Path) -> Result<(), Error> {
    let context = || {
        format!(
            "Extracting '{}' to '{}'",
            src.to_string_lossy(),
            dest.to_string_lossy()
        )
    };

    let mut file = file_open(src).e_context(context)?;
    let mut buf = [0u8; 6];
    file.read_exact(&mut buf).e_context(context)?;
    drop(file);

    if infer::archive::is_xz(&buf) {
        extract_tar_xz(src, dest).e_context(context)
    } else if infer::archive::is_gz(&buf) {
        extract_tar_gz(src, dest).e_context(context)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unknown archive / file type",
        )
        .throw(context()))
    }
}

/// Extracts a `tar` `xz` archive
/// # Arguments
/// * `src` - The path to the source archive file
/// * `dest` - The destination directory to extract the archive to
pub fn extract_tar_xz(src: &Path, dest: &Path) -> Result<(), Error> {
    let context = || {
        format!(
            "Extracting tar xz '{}' to '{}'",
            src.to_string_lossy(),
            dest.to_string_lossy()
        )
    };

    let file = file_open(src).e_context(context)?;

    let xz = xz::read::XzDecoder::new(file);
    let mut tar = tar::Archive::new(xz);

    tar.unpack(dest).e_context(context)
}

/// Extracts a `tar` `gz` archive
/// # Arguments
/// * `src` - The path to the source archive file
/// * `dest` - The destination directory to extract the archive to
pub fn extract_tar_gz(src: &Path, dest: &Path) -> Result<(), Error> {
    let context = || {
        format!(
            "Extracting tar gz '{}' to '{}'",
            src.to_string_lossy(),
            dest.to_string_lossy()
        )
    };

    let file = file_open(src).e_context(context)?;

    let gz = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);

    tar.unpack(dest).e_context(context)
}
