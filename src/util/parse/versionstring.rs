//! Parsing utilities for version strings

use serde::{Deserializer, Serializer};

/// A version string that can be deserialized
#[derive(Debug, Clone)]
pub struct VersionString {
    pub name: String,
    pub version: String,
    pub pkgver: u32,
}

impl<'de> serde::Deserialize<'de> for VersionString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CustomStringVisitor;

        impl<'de> serde::de::Visitor<'de> for CustomStringVisitor {
            type Value = VersionString;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string in the format 'sth@sflkn/sdan'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut parts = value.splitn(3, |c| c == '@' || c == '/');
                let name = parts
                    .next()
                    .ok_or_else(|| E::custom("Missing name or '@' delimiter"))?
                    .to_string();
                let version = parts
                    .next()
                    .ok_or_else(|| E::custom("Missing part after '@' or missing '@' after name"))?
                    .to_string();
                let pkgver = parts
                    .next()
                    .ok_or_else(|| {
                        E::custom("Missing part after '/' or missing '/' after version")
                    })?
                    .to_string();

                let pkgver = pkgver.parse::<u32>().map_err(|_| {
                    E::invalid_value(
                        serde::de::Unexpected::Str(&pkgver),
                        &"a string representing a u32",
                    )
                })?;

                Ok(VersionString {
                    name,
                    version,
                    pkgver,
                })
            }
        }

        deserializer.deserialize_str(CustomStringVisitor)
    }
}

// Implement custom serialization for the struct
impl serde::Serialize for VersionString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut string_repr = String::new();
        // Customize the serialization format as needed
        string_repr.push_str(&self.name);
        string_repr.push('@');
        string_repr.push_str(&self.version);
        string_repr.push('/');
        string_repr.push_str(&self.pkgver.to_string());

        serializer.serialize_str(&string_repr)
    }
}
