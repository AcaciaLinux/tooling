use std::path::{Path, PathBuf};

use crate::{
    error::Error,
    files::formula::FormulaFile,
    util::fs::Directory,
    validators::{
        dependencies_from_validation_result, indexed_package::FileValidationResult, ValidationInput,
    },
};

use super::{CorePackage, DependencyProvider, IndexedPackage, PackageInfo, PathPackage};

/// A package that has been built by the builder and is now ready to be validated
#[derive(Clone, Debug)]
pub struct BuiltPackage {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub description: String,

    pub formula: FormulaFile,

    pub dependencies: Vec<PackageInfo>,

    pub path: PathBuf,

    pub index: Directory,
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
    ) -> Result<(Self, Vec<FileValidationResult>), Error> {
        let index = Directory::index(&path.join("data"), true, false)?;

        // Construct a temporary self
        let mut self_ = Self {
            name: src.package.name.clone(),
            version: src.package.version.clone(),
            arch,
            description: src.package.description.clone(),

            formula: src,

            dependencies: Vec::new(),

            path: path.to_owned(),

            index,
        };

        // Validate the package
        let val_res = self_.validate(validation_input);

        // Set the dependencies
        self_.dependencies = dependencies_from_validation_result(&val_res)
            .iter()
            .map(|d| (*d).clone())
            .collect();

        Ok((self_, val_res))
    }
}

impl CorePackage for BuiltPackage {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_version(&self) -> &str {
        &self.version
    }

    fn get_arch(&self) -> &str {
        &self.arch
    }
}

impl IndexedPackage for BuiltPackage {
    fn get_index(&self) -> &Directory {
        &self.index
    }
}

impl PathPackage for BuiltPackage {
    fn get_real_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl DependencyProvider for BuiltPackage {
    fn get_dependencies(&self) -> Vec<&PackageInfo> {
        self.dependencies.iter().collect()
    }
}
