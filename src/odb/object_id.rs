use std::fmt::Display;

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
    /// Returns the ID (the digest) as a vector of binary data
    pub fn id(&self) -> Vec<u8> {
        match self.inner {
            InnerObjectID::SHA256(v) => v.into(),
        }
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
