use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use log::debug;
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
};

use super::{
    odb_driver::FilesystemDriver, Home, Object, ObjectCompression, ObjectDB, ObjectID, ObjectType,
    Tree,
};

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

    /// The tree of files that is shipped with this formula
    pub tree: ObjectID,
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
    /// Parses and resolves a formula by resolving the following:
    /// - Dependencies
    /// - Files
    ///
    /// All of these things get inserted into the `object_db`
    /// to resolve all dependencies as object references.
    ///
    /// This will also insert the formula into the object database
    /// # Arguments
    /// * `formula_path` - The path to the formula file
    /// * `home` - The home to use for the resolving process
    /// * `build_architecture` - The architecture the formula is built for
    /// * `compression` - The compression method to use for inserting the objects
    pub fn parse_and_resolve(
        formula_path: &Path,
        home: &Home,
        build_architecture: Architecture,
        compression: ObjectCompression,
    ) -> Result<(Formula, Object), Error> {
        let formula: FormulaFile = toml::from_str(&fs::file_read_to_string(formula_path)?)
            .e_context(|| "Parsing formula source")?;

        let parent = formula_path
            .parent()
            .expect("Parent directory of formula file");

        let file_sources = formula.package.sources.clone().unwrap_or_default();
        let odb_driver = FilesystemDriver::new(home.object_db_path())?;
        let mut object_db = ObjectDB::init(Box::new(odb_driver)).ctx(|| "Opening object db")?;
        let temp_dir = home.get_temporary_directory();

        // If the formula has some supported architectures,
        // make sure the build architecture is in them
        let architecture = match formula.package.get_architectures() {
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

        let mut tree = Tree::index(parent, &mut object_db, compression, true)
            .ctx(|| "Indexing formula files")?;

        for source in file_sources {
            let url = source.get_url(&formula.package);
            let dest = PathBuf::from(source.get_dest(&formula.package));

            let path = temp_dir.join(&dest);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).ctx(|| "Creating source parent directory")?;
            }

            download_to_file(
                &url,
                &path,
                &format!("Fetching source {}", dest.str_lossy()),
                true,
            )?;
        }

        let sources_tree = Tree::index(&temp_dir, &mut object_db, compression, true)
            .ctx(|| "Creating sources tree")?;
        tree.merge(sources_tree);

        let tree_obj = tree
            .insert_into_odb(&mut object_db, compression, true)
            .ctx(|| "Inserting tree")?;

        let formula = Formula {
            name: formula.package.name,
            version: formula.package.version,
            description: formula.package.description,

            strip: formula.package.strip,
            arch: architecture,

            host_dependencies: resolve_packages(formula.package.host_dependencies),
            target_dependencies: resolve_packages(formula.package.target_dependencies),
            extra_dependencies: resolve_packages(formula.package.extra_dependencies),

            prepare: formula.package.prepare,
            build: formula.package.build,
            check: formula.package.check,
            package: formula.package.package,

            layout: formula.package.layout,
            tree: tree_obj.oid,
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

    /// Returns the `JSON` string for this formula
    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("Serialize formula file should never fail")
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
        let mut cursor = Cursor::new(self.json());

        let object = object_db.insert_stream(
            &mut cursor,
            ObjectType::AcaciaFormula,
            compression,
            true,
            vec![self.tree.clone()],
        )?;

        debug!(
            "Inserted formula {}@{} as {}",
            self.name, self.version, object.oid
        );

        Ok(object)
    }
}
