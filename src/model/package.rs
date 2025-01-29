use std::io::Cursor;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::{error::Error, model::ObjectType, util::architecture::Architecture};

use super::{Object, ObjectCompression, ObjectDB, ObjectID};

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

impl Package {
    /// Returns the `TOML` string for this package
    pub fn toml(&self) -> String {
        toml::to_string_pretty(self).expect("Serialize package file should never fail")
    }

    /// Returns the `JSON` string for this package
    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("Serialize package file should never fail")
    }

    /// Inserts this package into `object_db`
    /// # Arguments
    /// * `object_db` - The objet db to insert the package into
    /// * `compression` - The compression to apply for inserting
    pub fn insert(
        &self,
        object_db: &mut ObjectDB,
        compression: ObjectCompression,
    ) -> Result<Object, Error> {
        let mut cursor = Cursor::new(self.json());

        let mut dependencies = vec![self.tree.clone()];
        for d in &self.dependencies {
            dependencies.push(d.clone());
        }

        let object = object_db.insert_stream(
            &mut cursor,
            ObjectType::AcaciaFormula,
            compression,
            dependencies,
        )?;

        debug!(
            "Inserted package {}@{} as {}",
            self.name, self.version, object.oid
        );

        Ok(object)
    }
}
