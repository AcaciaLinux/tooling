//! Utilities for working with architectures

use crate::error::Error;
use serde::{Deserialize, Serialize};

use crate::error::ErrorExt;

/// An architecture description containing a main architecture and subarchitectures
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Architecture {
    /// The main architecture
    pub arch: String,
    /// Subarchitectures such as extensions
    pub subarchs: Vec<String>,
}

/// Returns `true` if `subset` is a subset of `of` by taking each element
/// in `subset` and checking if it exists in `of`
/// # Arguments
/// * `subset` - The subset to ensure is in `of`
/// * `of` - The superset to contain the elements of `subset`
pub fn is_subset<T: PartialEq>(subset: &[T], of: &[T]) -> bool {
    for e in subset {
        if !of.contains(e) {
            return false;
        }
    }
    true
}

impl Architecture {
    /// Creates a new `Architecture` from the two arguments
    /// # Arguments
    /// * `arch` - The main architecture
    /// * `subarchs` - The sub-architectures
    pub fn new(arch: String, subarchs: Vec<String>) -> Self {
        Self { arch, subarchs }
    }

    /// Creates a new `Architecture` from only the main architecture and no subarchs
    /// # Arguments
    /// * `arch` - The main architecture
    pub fn new_arch(arch: String) -> Self {
        Self {
            arch,
            subarchs: Vec::new(),
        }
    }

    /// Creates a new architecture by detecting it using the `uname` crete
    pub fn new_uname() -> Result<Self, Error> {
        let info = uname::uname().e_context(|| "Determining host architecture".to_string())?;

        Ok(Self::new(info.machine, Vec::new()))
    }

    /// Checks if this architecture can run on `on`.
    ///
    /// This will check if `self` is a subset of `on`
    pub fn can_run_on(&self, on: &Architecture) -> bool {
        // If the main architectures don't match, we can't run
        if self.arch != on.arch {
            return false;
        }

        is_subset(&self.subarchs, &on.subarchs)
    }

    /// Checks if this architecture supports hosting `other`.
    ///
    /// This will check if `other` is a subset of `self`
    pub fn can_host(&self, other: &Architecture) -> bool {
        // If the main architectures don't match, we can't run
        if self.arch != other.arch {
            return false;
        }

        is_subset(&other.subarchs, &self.subarchs)
    }
}
