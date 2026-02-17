use std::{
    fmt::Display,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{
    error::{ALErrorExt, ALResult},
    str,
    util::hash::{sha256_file, sha256_stream},
};

/// The SHA256 object id is identified by a ASCII 'S', 0x53
const OID_ID_SHA256: u8 = 0x53;

/// An object ID uniquely identifies an object by its hash digest
pub struct ObjectID {
    inner: InnerObjectID,
}

enum InnerObjectID {
    /// The SHA256 variant of a object ID
    SHA256([u8; 33]),
}

impl ObjectID {
    /// Derives a SHA256 Object ID from the file at
    /// the provided path by reading and hashing
    /// all of its contents
    /// # Arguments
    /// * `file` - The file to be hashed to an object id
    pub fn from_sha256_file(file: &Path) -> ALResult<Self> {
        let ctx = str!("Deriving Object ID from stream");

        Ok(sha256_file(file).ctx(ctx)?.into())
    }

    /// Derives a SHA256 Object ID from the provided
    /// stream by reading from it and hashing the contents
    ///
    /// This function does not respect the offset the stream
    /// is currently at, so if this is a file, the position
    /// is ignored and hashing simply starts from the current
    /// point in the stream
    /// # Arguments
    /// * `stream` - The stream to has into an object id
    pub fn from_sha256_stream<R: Read>(stream: &mut R) -> ALResult<Self> {
        let ctx = str!("Deriving Object ID from stream");

        Ok(sha256_stream(stream).ctx(ctx)?.into())
    }

    /// Returns the ID (the digest) as a vector of binary data
    pub fn id(&self) -> Vec<u8> {
        match self.inner {
            InnerObjectID::SHA256(v) => v.into(),
        }
    }

    /// Encodes this object id to a hex string
    pub fn to_hex_str(&self) -> String {
        hex::encode(self.id())
    }

    /// Constructs a path for this object id and a depth:
    ///
    /// - `abcdef` => `abcdef` (depth = 0)
    /// - `abcdef` => `ab/abcdef` (depth = 1)
    /// - `abcdef` => `ab/cd/abcdef` (depth = 2)
    /// # Arguments
    /// * `depth` - The depth to split the id into
    pub fn to_path(&self, depth: usize) -> PathBuf {
        let oid_string = self.to_hex_str();

        let mut path = PathBuf::new();
        let mut oid = oid_string.as_str();

        for _ in 0..depth {
            let split = oid.split_at(2);
            path.push(split.0);
            oid = split.1;
        }

        path.join(oid_string)
    }
}

impl From<sha2::digest::Output<sha2::Sha256>> for ObjectID {
    fn from(value: sha2::digest::Output<sha2::Sha256>) -> Self {
        let digest: [u8; 32] = value.into();
        let mut inner = [0u8; 33];
        inner[0] = OID_ID_SHA256;
        inner[1..].copy_from_slice(&digest);
        Self {
            inner: InnerObjectID::SHA256(inner),
        }
    }
}

impl Display for ObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.id()))
    }
}
