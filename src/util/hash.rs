//! Utilities for dealing with hashes
use std::{
    fs::File,
    io::{copy, Read},
    path::Path,
};

use sha2::{digest::Output, Digest, Sha256};

use crate::error::{Error, ErrorExt};

/// Hashes the supplied string
/// # Arguments
/// * `string` - The string to hash
pub fn hash_string(string: &str) -> Output<Sha256> {
    let mut hasher = Sha256::new();
    hasher.update(string);
    hasher.finalize()
}

/// Hashes a file
/// # Arguments
/// * `file` - The path to the file to hash
pub fn hash_file(file: &Path) -> Result<Output<Sha256>, Error> {
    let context = || format!("Hashing file {}", file.to_string_lossy());

    let mut f = File::open(file).e_context(context)?;
    let mut hasher = Sha256::new();
    copy(&mut f, &mut hasher).e_context(context)?;

    Ok(hasher.finalize())
}

/// Hashes a readable stream
/// # Arguments
/// * `input` - The stream to hash
pub fn hash_stream<R: Read>(input: &mut R) -> Result<Output<Sha256>, Error> {
    let mut hasher = Sha256::new();

    copy(input, &mut hasher).e_context(|| "Hashing stream")?;

    Ok(hasher.finalize())
}
