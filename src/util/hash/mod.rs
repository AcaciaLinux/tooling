//! Utilities for hashing things
use std::{fs::File, io::Read, path::Path};

use sha2::{Digest, Sha256, digest::Output};

use crate::{
    error::{ALErrorExt, ALResult},
    fs::PathExt,
    str,
};

/// SHA256 Hashes the supplied string
/// # Arguments
/// * `string` - The string to hash
pub fn sha256_string(string: &str) -> Output<Sha256> {
    let mut hasher = Sha256::default();
    hasher.update(string);
    hasher.finalize()
}

/// SHA256 Hashes a file
/// # Arguments
/// * `file` - The path to the file to hash
pub fn sha256_file(file: &Path) -> ALResult<Output<Sha256>> {
    let ctx = str!("SHA256 Hashing {}", file.str_lossy());
    let mut f = File::open(file).ctx(ctx)?;
    sha256_stream(&mut f).ctx(ctx)
}

/// SHA256 Hashes a readable stream
/// # Arguments
/// * `input` - The stream to hash
pub fn sha256_stream<R: Read>(input: &mut R) -> ALResult<Output<Sha256>> {
    let mut hasher = Sha256::new();

    std::io::copy(input, &mut hasher).ctx(str!("SHA256 Hashing stream"))?;

    Ok(hasher.finalize())
}
