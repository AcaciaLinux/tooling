//! Data structures to parse a package metadata file
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::package::{CorePackage, DependencyProvider, DescribedPackage};

/// The current version for the package meta file
static CUR_VERSION: u32 = 1;

/// The contents of a package meta file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetaFile {
    /// The version of the file
    pub version: u32,

    /// The package this file describes
    pub package: PackageMeta,
}

/// A package in the package metadata file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageMeta {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub description: String,

    pub dependencies: HashMap<String, PackageMetaDependency>,
}

/// A dependency of the package in the package metadata file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageMetaDependency {
    pub arch: String,
    pub req_version: String,
    pub lnk_version: Option<String>,
}

impl PackageMetaFile {
    /// Generates a package metadata file from a package that meets the requirements
    /// # Arguments
    /// * `in_package` - The package to generate this file from
    pub fn from_package<T>(in_package: &T) -> Self
    where
        T: CorePackage + DescribedPackage + DependencyProvider,
    {
        // Take all dependencies and make their versions the required and the linked ones
        let mut dependencies = HashMap::new();
        for dep in in_package.get_dependencies() {
            let dep = dep.clone();
            dependencies.insert(
                dep.name,
                PackageMetaDependency {
                    arch: dep.arch,
                    req_version: dep.version.clone(),
                    lnk_version: Some(dep.version),
                },
            );
        }

        // Create the package metadata
        let package = PackageMeta {
            name: in_package.get_name().to_owned(),
            version: in_package.get_version().to_owned(),
            arch: in_package.get_arch().to_owned(),
            description: in_package.get_description().to_string(),
            dependencies,
        };

        Self {
            version: CUR_VERSION,
            package,
        }
    }
}
