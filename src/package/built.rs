use std::path::{Path, PathBuf};

use crate::{
    dist_dir,
    error::Error,
    files::formula::FormulaFile,
    util::fs::Directory,
    validators::{
        dependencies_from_validation_result, indexed_package::FileValidationResult, ValidationInput,
    },
};

use super::{
    index::{IndexCollection, IndexedPackageIndex},
    ArchitecturePackage, BuildIDProvider, CorePackage, DependencyProvider, DescribedPackage,
    IndexedPackage, NameVersionPackage, NamedPackage, PackageInfo, PathPackage, VersionedPackage,
};

/// A package that has been built by the builder and is now ready to be validated
#[derive(Clone, Debug)]
pub struct BuiltPackage {
    pub name: String,
    pub version: String,
    pub pkgver: u32,
    pub arch: String,
    pub description: String,

    pub formula: FormulaFile,

    pub dependencies: Vec<PackageInfo>,

    pub path: PathBuf,

    pub index: Directory,
    pub build_id: String,
}

impl BuiltPackage {
    /// Constructs a built package from a formula package, the built architecture and the index of its file contents by validating the package and adding the dependencies
    ///
    /// This will validate the package using the `validation_input`
    ///
    /// This will **not** unwind symlinks to prevent double treatment of files
    /// # Arguments
    /// * `src` - The source `FormulaPackage` to construct this package from
    /// * `arch` - The architecture the package has been built for
    /// * `path` - The path to the package, containing the `data/` directory
    /// * `validation_input` - The input to the validation functions
    /// # Returns
    /// The `BuiltPackage` and a vector of validation errors, an error if something has gone horribly wrong
    pub fn from_formula_validate(
        src: FormulaFile,
        arch: String,
        path: &Path,
        validation_input: &ValidationInput,
        build_id: String,
    ) -> Result<(Self, Vec<FileValidationResult>), Error> {
        let info = PackageInfo::from_package_arch(&src.package, arch.clone());

        let package_archive_dir = path.join("data");
        let archive_index = Directory::index(&package_archive_dir, true, false)?;

        // Construct a temporary self
        let mut self_ = Self {
            name: src.package.name.clone(),
            version: src.package.version.clone(),
            pkgver: src.package.pkgver,
            arch: arch.clone(),
            description: src.package.description.clone(),

            formula: src,

            dependencies: Vec::new(),

            path: path.to_owned(),

            index: archive_index,
            build_id: build_id.clone(),
        };

        let val_res = {
            // Create a dummy package to use as an installed package
            // This allows the validators to find files from 'self'
            let dummy_package = {
                let package_root_dir = path
                    .join("data")
                    .join(info.get_path(&dist_dir()))
                    .join("root");

                let mut dummy_self = self_.clone();
                dummy_self.index = Directory::index(&package_root_dir, true, true)?;
                dummy_self
            };

            // Create an indexed package index with the current package in int
            let index = IndexedPackageIndex::new(vec![&dummy_package]);
            // Create a collection of the previous index and the created one
            let collection = IndexCollection {
                indices: vec![&index, validation_input.package_index],
            };

            // Construct a new validation input
            let val_input = ValidationInput {
                package_index: &collection,
                strip: validation_input.strip,
            };

            // Validate the package
            self_.validate(&val_input)
        };

        // Set the dependencies
        self_.dependencies = dependencies_from_validation_result(&val_res)
            .iter()
            .map(|d| (*d).clone())
            .collect();
        // Drop dependencies on 'self'
        self_.dependencies.retain(|p| p.name != self_.name);

        Ok((self_, val_res))
    }
}

impl NamedPackage for BuiltPackage {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl VersionedPackage for BuiltPackage {
    fn get_version(&self) -> &str {
        &self.version
    }
    fn get_pkgver(&self) -> u32 {
        self.pkgver
    }
}

impl ArchitecturePackage for BuiltPackage {
    fn get_arch(&self) -> &str {
        &self.arch
    }
}

impl NameVersionPackage for BuiltPackage {}

impl CorePackage for BuiltPackage {}

impl IndexedPackage for BuiltPackage {
    fn get_index(&self) -> &Directory {
        &self.index
    }
}

impl DescribedPackage for BuiltPackage {
    fn get_description(&self) -> &str {
        &self.description
    }
}

impl PathPackage for BuiltPackage {
    fn get_real_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl BuildIDProvider for BuiltPackage {
    fn get_build_id(&self) -> &str {
        &self.build_id
    }
}

impl DependencyProvider for BuiltPackage {
    fn get_dependencies(&self) -> Vec<&PackageInfo> {
        self.dependencies.iter().collect()
    }
}
