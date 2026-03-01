//! Configuration of the Acacia tooling, including file,
//! extension and feature definitions

/// The suffix (or file extension) for a Acacia object
pub const SUFFIX_ACACIA_OBJECT: &str = "aobj";

/// The suffix (or file extension) for a raw Acacia object
pub const SUFFIX_RAW_OBJECT: &str = "robj";

/// The size of the buffer used when hashing a stream or a file in the [util::hash](crate::util::hash) module.
pub(crate) const HASH_COPY_BUFFER_SIZE: usize = 4096;
