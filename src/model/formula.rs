use std::{io::Cursor, path::PathBuf};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    error::{architecture::ArchitectureError, Error, ErrorExt, ErrorType},
    files::formulafile::FormulaFile,
    util::{
        architecture::Architecture,
        download::download_to_file,
        fs::{self, PathUtil},
        parse::versionstring::VersionString,
    },
    ODB_DEPTH,
};

use super::{Home, Object, ObjectCompression, ObjectDB, ObjectID};

/// A resolved formula that uniquely describes a package's
/// build instructions to be stored in the object database.
#[derive(Deserialize, Serialize, Debug)]
pub struct Formula {
    /// The name of the package
    pub name: String,
    /// The version of the package
    pub version: String,
    /// A short description of the package's contents
    pub description: String,

    /// Whether the package's binaries should be stripped
    /// using the `strip` command
    pub strip: bool,

    /// The architecture the package is built for
    pub arch: Option<Architecture>,

    /// The dependencies that are required on the building
    /// side of the package
    pub host_dependencies: Vec<ObjectID>,
    /// The dependencies that are needed at build-time that
    /// the resulting binaries link against
    pub target_dependencies: Vec<ObjectID>,
    /// Dependencies that are not required at build-time,
    /// but on runtime and are not automatically picked up
    /// by the dependency checker
    pub extra_dependencies: Vec<ObjectID>,

    /// The instructions for the `prepare` step
    pub prepare: Option<String>,
    /// The instructions for the `build` step
    pub build: Option<String>,
    /// The instructions for the `check` step
    pub check: Option<String>,
    /// The instructions for the `package` step
    pub package: Option<String>,

    /// The layout describing the purposes and
    /// special directories within the package root
    pub layout: IndexMap<String, Vec<String>>,

    /// The files that are shipped with this formula
    /// including the downloaded source files
    pub files: IndexMap<PathBuf, ObjectID>,
}

/// Helper function to resolve an optional vector of
/// package strings to a vector of object ids
/// # Arguments
/// * `packages` - The packages to resolve
fn resolve_packages(packages: Option<Vec<VersionString>>) -> Vec<ObjectID> {
    let oids = Vec::new();

    for _ in packages.unwrap_or_default() {
        todo!("Implement package resolving")
    }

    oids
}

impl FormulaFile {
    /// Resolves a formula by resolving the following:
    /// - Dependencies
    /// - Files
    ///
    /// All of these things get inserted into the `object_db`
    /// to resolve all dependencies as object references.
    ///
    /// This will also insert the formula into the object database
    /// # Arguments
    /// * `home` - The home to use for the resolving process
    /// * `parent_dir` - The parent directory of the formula file
    /// * `build_architecture` - The architecture the formula is built for
    /// * `compression` - The compression method to use for inserting the objects
    pub fn resolve(
        self,
        home: &Home,
        parent: PathBuf,
        build_architecture: Architecture,
        compression: ObjectCompression,
    ) -> Result<(Formula, Object), Error> {
        let mut files: IndexMap<PathBuf, ObjectID> = IndexMap::new();
        let file_sources = self.package.sources.clone().unwrap_or_default();
        let mut object_db = ObjectDB::init(home.object_db_path(), ODB_DEPTH)?;

        // If the formula has some supported architectures,
        // make sure the build architecture is in them
        let architecture = match self.package.get_architectures() {
            None => Ok(None),
            Some(archs) => {
                let supported_archs: Vec<&Architecture> = archs
                    .iter()
                    .filter(|a| a.can_run_on(&build_architecture))
                    .collect();

                if supported_archs.is_empty() {
                    Err(Error::new(ErrorType::Architecture(
                        ArchitectureError::NotSupported {
                            arch: build_architecture,
                            supported: archs,
                        },
                    )))
                } else {
                    Ok(Some(build_architecture))
                }
            }
        }
        .e_context(|| "Resolving formula architecture")?;

        for source in file_sources {
            let url = source.get_url(&self.package);
            let dest = PathBuf::from(source.get_dest(&self.package));

            let tmp_path = home.get_temp_file_path();
            download_to_file(
                &url,
                &tmp_path,
                &format!("Fetching source {}", dest.str_lossy()),
                true,
            )?;

            let object = object_db.insert_file(&tmp_path, compression, true)?;

            files.insert(dest, object.oid);
        }

        let mut results = Vec::new();

        fs::walk_dir(&parent, true, &mut |entry| {
            results.push((
                entry.path(),
                object_db.insert_file(&entry.path(), compression, true),
            ));
            true
        })
        .e_context(|| "Walking formula parent directory")?;

        for (path, object) in results {
            let object = object?;

            files.insert(path, object.oid);
        }

        let formula = Formula {
            name: self.package.name,
            version: self.package.version,
            description: self.package.description,

            strip: self.package.strip,
            arch: architecture,

            host_dependencies: resolve_packages(self.package.host_dependencies),
            target_dependencies: resolve_packages(self.package.target_dependencies),
            extra_dependencies: resolve_packages(self.package.extra_dependencies),

            prepare: self.package.prepare,
            build: self.package.build,
            check: self.package.check,
            package: self.package.package,

            layout: self.package.layout,
            files,
        };

        let object = formula.insert(&mut object_db, compression)?;

        Ok((formula, object))
    }
}

impl Formula {
    /// Returns the `TOML` string for this formula
    pub fn toml(&self) -> String {
        toml::to_string_pretty(self).expect("Serialize formula file should never fail")
    }

    /// Inserts this formula into `object_db`
    /// # Arguments
    /// * `object_db` - The objet db to insert the formula into
    /// * `compression` - The compression to apply for inserting
    pub fn insert(
        &self,
        object_db: &mut ObjectDB,
        compression: ObjectCompression,
    ) -> Result<Object, Error> {
        let mut cursor = Cursor::new(self.toml());
        object_db.insert_stream(&mut cursor, compression, true)
    }
}
