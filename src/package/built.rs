use crate::{files::formula::FormulaPackage, util::fs::Directory};

use super::{CorePackage, FQPackage, IndexedPackage};

/// A package that has been built by the builder and is now ready to be validated
#[derive(Clone)]
pub struct BuiltPackage {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub real_version: u32,
    pub description: String,

    pub index: Directory,
}

impl BuiltPackage {
    /// Constructs a built package from a formula package, the built architecture and the index of its file contents
    pub fn from_formula(src: FormulaPackage, arch: String, index: Directory) -> Self {
        Self {
            name: src.name,
            version: src.version,
            arch,
            real_version: src.real_version,
            description: src.description,
            index,
        }
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

impl FQPackage for BuiltPackage {
    fn get_real_version(&self) -> u32 {
        self.real_version
    }
}

impl IndexedPackage for BuiltPackage {
    fn get_index(&self) -> &Directory {
        &self.index
    }
}
