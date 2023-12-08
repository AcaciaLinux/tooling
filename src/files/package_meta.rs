//! Data structures to parse a package metadata file
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The contents of a package meta file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    /// The version of the file
    pub version: u32,

    /// The package this file describes
    pub package: Package,
}

/// A package in the package metadata file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub real_version: u32,
    pub description: String,

    pub dependencies: HashMap<String, Dependency>,
}

/// A dependency of the package in the package metadata file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dependency {
    pub arch: String,
    pub req_version: String,
    pub lnk_version: Option<String>,
}
