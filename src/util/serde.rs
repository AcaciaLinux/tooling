//! Utilities for working with serde

use base64::Engine;
use serde::{de, Deserialize, Deserializer};

use crate::BASE64_ENGINE;

/// Deserializes a `Vec<u8>` from a base 64 string using [crate::BASE64_ENGINE]
pub fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the base64 string
    let base64_string: &str = Deserialize::deserialize(deserializer)?;

    // Decode base64 string into a byte vector
    let decoded_bytes = BASE64_ENGINE
        .decode(base64_string)
        .map_err(|err| de::Error::custom(format!("base64 decoding error: {}", err)))?;

    Ok(decoded_bytes)
}
