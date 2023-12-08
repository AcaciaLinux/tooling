//! Tools for validating packages and files to be able to work in the AcaciaLinux system

pub mod elf;
pub mod indexed_package;
pub mod scripts;

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
pub enum ValidatorAction {
    /// Perform an action on a `ELF` file
    ELF(elf::ELFAction),
    /// Perform an action on a `Script` file
    Script(scripts::ScriptAction),
}

impl std::fmt::Display for ValidatorAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ELF(action) => action.fmt(f),
            Self::Script(action) => action.fmt(f),
        }
    }
}
