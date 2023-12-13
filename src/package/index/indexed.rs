use crate::package::IndexedPackage;

use super::PackageIndex;

/// An index of packages that implement the `IndexedPackage` trait
pub struct IndexedPackageIndex<'a> {
    /// The packages contained in this index
    packages: Vec<&'a dyn IndexedPackage>,
}

impl<'a> IndexedPackageIndex<'a> {
    /// Creates a new index using the supplied vector of values
    /// # Arguments
    /// * `packages` - The packages to house in this index
    pub fn new(packages: Vec<&'a dyn IndexedPackage>) -> Self {
        Self { packages }
    }

    /// Push a new package to this index
    /// # Arguments
    /// * `package` - The package to push
    pub fn push(&mut self, package: &'a dyn IndexedPackage) {
        self.packages.push(package)
    }
}

impl<'a> PackageIndex for IndexedPackageIndex<'a> {
    fn find_fs_entry(
        &self,
        entry: &crate::util::fs::SearchType,
    ) -> Option<(
        std::collections::LinkedList<std::ffi::OsString>,
        &dyn IndexedPackage,
    )> {
        for p in &self.packages {
            if let Some(found_entry) = p.get_index().find_entry(entry) {
                return Some((found_entry, *p));
            }
        }

        None
    }
}
