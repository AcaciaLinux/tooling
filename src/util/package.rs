//! Utilities for managing packages
use crate::files::package_index::IndexPackage;

mod installed;
pub use installed::*;

/// A provider for `IndexPackage`s
pub trait PackageIndexProvider<'a> {
    /// Returns the array of `IndexPackage`s the struct provides
    fn get_packages(&'a self) -> &'a [IndexPackage];
}
