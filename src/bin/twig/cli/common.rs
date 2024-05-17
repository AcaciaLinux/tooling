use std::fmt::Display;

use clap::ValueEnum;
use tooling::model::ObjectCompression;

/// Compression types available for the tooling
#[derive(ValueEnum, Clone)]
pub enum Compression {
    /// No compression
    None,
    /// Apply XZ compression
    Xz,
}

impl Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "none",
                Self::Xz => "xz",
            }
        )
    }
}

impl From<Compression> for ObjectCompression {
    fn from(value: Compression) -> Self {
        match value {
            Compression::None => ObjectCompression::None,
            Compression::Xz => ObjectCompression::Xz,
        }
    }
}
