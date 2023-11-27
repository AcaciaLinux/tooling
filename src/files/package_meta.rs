//! Data structures to parse a package metadata file
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The contents of a package meta file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    /// The version of the file
    pub version: u32,

    /// The package this file describes
    #[serde(deserialize_with = "deserialize_package")]
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

    pub dependencies: Vec<Dependency>,
}

/// A dependency of the package in the package metadata file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dependency {
    pub name: String,
    pub req_version: String,
    pub lnk_version: String,
}

/// Deserializes a `PackageMeta` struct from a deserializer
fn deserialize_package<'de, D>(deserializer: D) -> Result<Package, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct PackageRaw {
        pub name: String,
        pub version: String,
        pub arch: String,
        pub real_version: u32,
        pub description: String,

        pub dependencies: HashMap<String, DependencyRaw>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct DependencyRaw {
        pub req_version: String,
        pub lnk_version: String,
    }

    let package = PackageRaw::deserialize(deserializer)?;

    let dependencies: Vec<Dependency> = package
        .dependencies
        .into_iter()
        .map(|m| Dependency {
            name: m.0.to_owned(),
            req_version: m.1.req_version,
            lnk_version: m.1.lnk_version,
        })
        .collect();

    Ok(Package {
        name: package.name,
        version: package.version,
        arch: package.arch,
        real_version: package.real_version,
        description: package.description,
        dependencies,
    })
}
