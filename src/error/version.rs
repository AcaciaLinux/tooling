/// An error that arises when versions are not supported
#[derive(Debug)]
pub enum VersionError {
    /// The version of object in this variant
    /// is not supported
    ObjectVersionNotSupported(u8),

    /// The magic sequence of a file is unknown / not supported
    ObjectMagicNotSupported([u8; 4]),
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectVersionNotSupported(version) => {
                write!(f, "Object version {version} is not supported",)
            }
            Self::ObjectMagicNotSupported(magic) => {
                write!(f, "Object magic {:?} is not supported", magic)
            }
        }
    }
}
