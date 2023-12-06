//! Data structures to parse a package index file
use crate::package::{CorePackage, PackageIndexProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The contents of a package index file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageIndexFile {
    pub version: u32,

    #[serde(deserialize_with = "deserialize_packages")]
    pub package: Vec<IndexPackage>,
}

/// A package in the index file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPackage {
    pub name: String,
    pub version: String,
    pub arch: String,
}

impl CorePackage for IndexPackage {
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

impl PackageIndexProvider for PackageIndexFile {
    fn get_packages(&self) -> &[IndexPackage] {
        &self.package
    }

    fn find_package(&self, name: &str) -> Option<&IndexPackage> {
        self.package.iter().find(|p| p.name == name)
    }
}

/// Deserializes a `PackageMeta` struct from a deserializer
fn deserialize_packages<'de, D>(deserializer: D) -> Result<Vec<IndexPackage>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Wrapper {
        package: HashMap<String, IndexPackageRaw>,
    }

    #[derive(Deserialize)]
    struct IndexPackageRaw {
        pub version: String,
        pub arch: String,
    }

    let wrapper = Wrapper::deserialize(deserializer)?;

    let packages: Vec<IndexPackage> = wrapper
        .package
        .into_iter()
        .map(|m| IndexPackage {
            name: m.0.to_owned(),
            version: m.1.version,
            arch: m.1.arch,
        })
        .collect();

    Ok(packages)
}
