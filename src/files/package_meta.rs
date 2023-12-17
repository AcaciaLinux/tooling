//! Data structures to parse a package metadata file
use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::package::{
    BuildIDProvider, CorePackage, DependencyProvider, DescribedPackage, IndexedPackage,
};

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
    pub pkgver: u32,
    pub arch: String,
    pub description: String,

    pub build_id: String,

    pub dependencies: HashMap<String, PackageMetaDependency>,

    #[serde(default)]
    pub executable_dirs: Vec<PathBuf>,

    #[serde(default)]
    pub library_dirs: Vec<PathBuf>,
}

/// A dependency of the package in the package metadata file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageMetaDependency {
    pub arch: String,
    pub req_version: PackageMetaDependencyVersion,
    pub lnk_version: Option<PackageMetaDependencyVersion>,
}

/// A version of a package dependency
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageMetaDependencyVersion {
    pub version: String,
    pub pkgver: u32,
}

impl PackageMetaFile {
    /// Generates a package metadata file from a package that meets the requirements
    /// # Arguments
    /// * `in_package` - The package to generate this file from
    pub fn from_package<T>(in_package: &T) -> Self
    where
        T: CorePackage + DescribedPackage + BuildIDProvider + DependencyProvider + IndexedPackage,
    {
        // Take all dependencies and make their versions the required and the linked ones
        let mut dependencies = HashMap::new();
        for dep in in_package.get_dependencies() {
            let dep = dep.clone();
            dependencies.insert(
                dep.name,
                PackageMetaDependency {
                    arch: dep.arch,
                    req_version: PackageMetaDependencyVersion {
                        version: dep.version.clone(),
                        pkgver: dep.pkgver,
                    },
                    lnk_version: Some(PackageMetaDependencyVersion {
                        version: dep.version,
                        pkgver: dep.pkgver,
                    }),
                },
            );
        }

        // Create the package metadata
        let package = PackageMeta {
            name: in_package.get_name().to_owned(),
            version: in_package.get_version().to_owned(),
            pkgver: in_package.get_pkgver(),
            arch: in_package.get_arch().to_owned(),
            description: in_package.get_description().to_string(),
            build_id: in_package.get_build_id().to_string(),
            dependencies,
            executable_dirs: in_package.get_executable_dirs().to_vec(),
            library_dirs: in_package.get_library_dirs().to_vec(),
        };

        Self {
            version: CUR_VERSION,
            package,
        }
    }
}
