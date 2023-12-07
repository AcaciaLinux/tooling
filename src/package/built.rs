use std::path::{Path, PathBuf};

use crate::{error::Error, files::formula::FormulaPackage, util::fs::Directory};

use super::{CorePackage, IndexedPackage, PathPackage};

/// A package that has been built by the builder and is now ready to be validated
#[derive(Clone)]
pub struct BuiltPackage {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub description: String,

    pub path: PathBuf,

    pub index: Directory,
}

impl BuiltPackage {
    /// Constructs a built package from a formula package, the built architecture and the index of its file contents
    /// # Arguments
    /// * `src` - The source `FormulaPackage` to construct this package from
    /// * `arch` - The architecture the package has been built for
    /// * `path` - The path to the package, containing the `data/` directory
    pub fn from_formula(src: FormulaPackage, arch: String, path: &Path) -> Result<Self, Error> {
        let index = Directory::index(&path.join("data"), true)?;

        Ok(Self {
            name: src.name,
            version: src.version,
            arch,
            description: src.description,

            path: path.to_owned(),

            index,
        })
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
