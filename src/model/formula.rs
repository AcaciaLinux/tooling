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

    /// The packages that originate from this formula
    pub packages: IndexMap<String, FormulaPackage>,

    /// The tree of files that is shipped with this formula
    pub tree: ObjectID,

    /// The instructions for the `prepare` step
    pub prepare: Option<String>,
    /// The instructions for the `build` step
    pub build: Option<String>,
    /// The instructions for the `check` step
    pub check: Option<String>,
    /// The instructions for the `package` step
    pub package: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FormulaPackage {
    /// A short description of the package's contents
    pub description: String,

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

    /// Whether the package's binaries should be stripped
    /// using the `strip` command
    pub strip: bool,

    /// The layout describing the purposes and
    /// special directories within the package root
    pub layout: IndexMap<String, Vec<String>>,
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

        let file_sources = formula.sources.clone().unwrap_or_default();
        let odb_driver = FilesystemDriver::new(home.object_db_path())?;
        let mut object_db = ObjectDB::init(Box::new(odb_driver)).ctx(|| "Opening object db")?;
        let temp_dir = home.get_temporary_directory();

        // If the formula has some supported architectures,
        // make sure the build architecture is in them
        let architecture = match formula.get_architectures() {
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

        let mut tree =
            Tree::index(parent, &mut object_db, compression).ctx(|| "Indexing formula files")?;

        for source in file_sources {
            let url = source.get_url(&formula);
            let dest = PathBuf::from(source.get_dest(&formula));

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

        let sources_tree =
            Tree::index(&temp_dir, &mut object_db, compression).ctx(|| "Creating sources tree")?;
        tree.merge(sources_tree);

        let tree_obj = tree
            .insert_into_odb(&mut object_db, compression)
            .ctx(|| "Inserting tree")?;

        let formula_clone = formula.clone();

        let host_dependencies = resolve_packages(formula.host_dependencies);
        let target_dependencies = resolve_packages(formula.target_dependencies);
        let extra_dependencies = resolve_packages(formula.extra_dependencies);

        let packages = parse_formula_packages(formula_clone, extra_dependencies.clone());

        let formula = Formula {
            name: formula.name,
            version: formula.version,
            description: formula.description,

            arch: architecture,

            host_dependencies,
            target_dependencies,
            extra_dependencies,

            tree: tree_obj.oid,

            packages,

            prepare: formula.prepare,
            build: formula.build,
            check: formula.check,
            package: formula.package,
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
            vec![self.tree.clone()],
        )?;

        debug!(
            "Inserted formula {}@{} as {}",
            self.name, self.version, object.oid
        );

        Ok(object)
    }
}

/// Parses and resolves the 'packages' field of a formula,
/// cloning the formula if there are no packages defined,
/// otherwise using the packages as overrides to the formula's defaults
/// # Arguments
/// * `formula_file` - The source from the parsed formula file
/// * `formula_extra_dependencies` - The extra dependencies inherited from the formula
fn parse_formula_packages(
    formula_file: FormulaFile,
    formula_extra_dependencies: Vec<ObjectID>,
) -> IndexMap<String, FormulaPackage> {
    let mut packages = IndexMap::new();

    let description = formula_file.description;
    let strip = formula_file.strip;
    let layout = formula_file.layout;

    if formula_file.packages.is_empty() {
        // If the 'packages' field is empty, we clone the formula as a default package
        let package = FormulaPackage {
            description,
            extra_dependencies: Vec::new(),
            prepare: None,
            build: None,
            check: None,
            package: None,
            strip,
            layout,
        };

        packages.insert(formula_file.name, package);
    } else {
        // Otherwise, iterate over the packages and override formula's presets
        for (source_name, source_package) in formula_file.packages {
            let mut layout = layout.clone();
            layout.extend(source_package.layout);

            let package_extra_dependencies = resolve_packages(source_package.extra_dependencies);
            let mut extra_dependencies = formula_extra_dependencies.clone();
            extra_dependencies.extend(package_extra_dependencies);
            extra_dependencies.dedup();

            let package = FormulaPackage {
                description: source_package.description.unwrap_or(description.clone()),
                extra_dependencies,
                prepare: source_package.prepare,
                build: source_package.build,
                check: source_package.check,
                package: source_package.package,
                strip: source_package.strip.unwrap_or(strip),
                layout,
            };

            packages.insert(source_name, package);
        }
    }

    packages
}
