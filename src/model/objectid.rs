use std::fmt::{Debug, Display};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sha2::{digest::Output, Sha256};

use crate::{
    error::ErrorExt,
    util::{Packable, Unpackable},
};

/// Represents an object id (hash)
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ObjectID {
    hash: Vec<u8>,
}

impl Debug for ObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectID")
            .field("hash", &self.to_string())
            .finish()
    }
}

impl ObjectID {
    /// Creates a new object id from a hash
    /// # Arguments
    /// * `hash` - The hash to take as a source
    pub fn new(hash: Vec<u8>) -> Self {
        Self { hash }
    }

    /// Decodes a object id from a hex string
    /// # Arguments
    /// * `hex_string` - The string to decode
    pub fn new_from_hex(hex_string: &str) -> Result<Self, hex::FromHexError> {
        let hash: Vec<u8> = hex::decode(hex_string)?;
        Ok(Self::new(hash))
    }

    /// Encodes this object id to a hex string
    pub fn to_hex_str(&self) -> String {
        hex::encode(&self.hash)
    }

    /// Returns the length of the object id in bytes
    pub fn len(&self) -> usize {
        self.hash.len()
    }

    /// Returns if the object id is empty
    pub fn is_empty(&self) -> bool {
        self.hash.is_empty()
    }

    /// Returns a byte slice of this object id
    pub fn bytes(&self) -> &[u8] {
        &self.hash
    }
}

impl From<Output<Sha256>> for ObjectID {
    fn from(value: Output<Sha256>) -> Self {
        Self::new(value.into_iter().collect())
    }
}

impl Display for ObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex_str())
    }
}

impl Serialize for ObjectID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex_str())
    }
}

impl<'de> Deserialize<'de> for ObjectID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hash: String = Deserialize::deserialize(deserializer)?;

        Self::new_from_hex(&hash)
            .map_err(|err| de::Error::custom(format!("hex decoding error: {}", err)))
    }
}

impl Packable for ObjectID {
    fn pack<W: std::io::prelude::Write>(&self, output: &mut W) -> Result<(), crate::error::Error> {
        output
            .write(self.bytes())
            .e_context(|| format!("Packing object id {}", self))?;
        Ok(())
    }
}

impl Unpackable for ObjectID {
    fn unpack<R: std::io::prelude::Read>(
        input: &mut R,
    ) -> Result<Option<Self>, crate::error::Error> {
        let mut buf = [0u8; 32];

        let len = input.read(&mut buf).e_context(|| "Unpacking Object ID")?;

        if len < buf.len() {
            Ok(None)
        } else {
            Ok(Some(Self {
                hash: Vec::from(buf),
            }))
        }
    }
}
