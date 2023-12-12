//! Dependency errors

/// An error when working with dependencies
#[derive(Debug)]
pub enum DependencyError {
    /// A dependency is unresolved
    Unresolved {
        arch: String,
        name: String,
        version: String,
        pkgver: u32,
    },
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unresolved {
                arch,
                name,
                version,
                pkgver,
            } => {
                write!(
                    f,
                    "Unresolved dependency {}/{}@{}/{}",
                    arch, name, version, pkgver
                )
            }
        }
    }
}
