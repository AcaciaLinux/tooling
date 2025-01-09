use serde::{Deserialize, Serialize};

use crate::util::architecture::Architecture;

use super::ObjectID;

/// A package in the AcaciaLinux ecosystem
#[derive(Deserialize, Serialize, Debug)]
pub struct Package {
    /// The name of the package
    pub name: String,
    /// The version of the package
    pub version: String,
    /// A short description of the package's contents
    pub description: String,

    /// The architecture the package is built for
    pub arch: Option<Architecture>,

    /// The dependencies of the package
    pub dependencies: Vec<ObjectID>,

    /// The OID of the formula the package originates from
    pub formula: ObjectID,

    /// The tree of files that is shipped with this formula
    pub tree: ObjectID,

    /// Whether the package is tainted or not.
    /// This tells a consumer if the package had additional
    /// lower directories when it was built
    pub tainted: bool,
}
