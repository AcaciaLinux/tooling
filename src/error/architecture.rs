//! Architecture errors

use crate::util::architecture::Architecture;

/// An error when working with dependencies
#[derive(Debug)]
pub enum ArchitectureError {
    /// The architecture `arch` cannot support the `possible`
    /// architectures
    NotSupported {
        /// The architecture that is not supported
        arch: Architecture,
        /// The supported architectures
        supported: Vec<Architecture>,
    },
}

impl std::fmt::Display for ArchitectureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSupported { arch, supported } => {
                let supported: Vec<String> = supported.iter().map(|s| s.to_string()).collect();
                write!(
                    f,
                    "Architecture {arch} is not supported ({})",
                    supported.join(", ")
                )
            }
        }
    }
}
