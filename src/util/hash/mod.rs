//! Utilities for hashing things
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use sha2::{Digest, Sha256, digest::Output};

use crate::{
    configuration::HASH_COPY_BUFFER_SIZE,
    error::{ALErrorExt, ALResult},
    fs::PathExt,
    str,
    util::io::NullSink,
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
/// The amount of data hashed (in bytes) and the resulting hash digest
pub fn sha256_file(file: &Path) -> ALResult<(usize, Output<Sha256>)> {
    let ctx = str!("SHA256 Hashing {}", file.str_lossy());
    let mut f = File::open(file).ctx(ctx)?;
    sha256_stream(&mut f, &mut NullSink::default()).ctx(ctx)
}

/// SHA256 Hashes a readable stream
/// # Arguments
/// * `input` - The stream to hash
/// * `output` - The stream to pass the hashed data to after it has been hashed
/// # Returns
/// The amount of data hashed (in bytes) and the resulting hash digest
pub fn sha256_stream<R: Read, W: Write>(
    input: &mut R,
    output: &mut W,
) -> ALResult<(usize, Output<Sha256>)> {
    let ctx = str!("SHA256 hashing stream");

    let mut hasher = Sha256::new();
    let mut buf = [0u8; HASH_COPY_BUFFER_SIZE];
    let mut total_len = 0;

    loop {
        let len = input.read(&mut buf).ctx(ctx)?;
        if len == 0 {
            break;
        }
        hasher.write(&buf[..len]).ctx(ctx)?;
        output.write(&buf[..len]).ctx(ctx)?;
        total_len += len;
    }

    Ok((total_len, hasher.finalize()))
}
