//! Tools for validating packages and files to be able to work in the AcaciaLinux system

pub mod elf;
pub mod indexed_package;

mod error;
pub use error::*;

use crate::{error::Error, package::InstalledPackageIndex};

/// The information required for a validator to work
pub struct ValidationInput<'a> {
    /// The index of packages a validator can use for finding packages and their contents
    pub package_index: &'a InstalledPackageIndex,
}

/// The result of a validation with multiple actions and (possibly) errors
pub struct ValidationResult<T> {
    /// The actions to perform according to the validator
    pub actions: Vec<T>,
    /// The errors that occured during validation
    pub errors: Vec<Error>,
}

/// The possible actions to take according to a validator
#[derive(Clone)]
pub enum ValidatorAction<'a> {
    /// Perform an action on a `ELF` file
    ELF(elf::ELFAction<'a>),
}

impl<'a> std::fmt::Display for ValidatorAction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ELF(action) => action.fmt(f),
        }
    }
}
