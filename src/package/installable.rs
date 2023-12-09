use std::path::{Path, PathBuf};

use crate::{
    assert_relative,
    error::{Error, ErrorExt},
    files::package_meta,
    util::{
        fs::{create_dir_all, create_symlink, Directory},
        parse::write_toml,
    },
};

use super::{
    ArchitecturePackage, BuildIDProvider, BuiltPackage, CorePackage, DependencyProvider,
    DescribedPackage, IndexedPackage, NameVersionPackage, NamedPackage, PackageInfo, PathPackage,
    VersionedPackage,
};

/// A package that is ready to be installed and deployed
#[derive(Debug)]
pub struct InstallablePackage {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub description: String,

    pub dependencies: Vec<PackageInfo>,

    pub path: PathBuf,

    pub index: Directory,
    build_id: String,
}

impl InstallablePackage {
    /// Creates a `InstallablePackage` from a `BuiltPackage` by doing the following:
    /// - Create the `package.toml` package metadata file in the following locations:
    ///     - Package root (`<archive path>/data/<built_package_path>`)
    ///     - Package archive root (`<archive path\>`)
    /// - Create the `link/` directory and populate it with the current dependencies
    /// # Arguments
    /// * `built_package` - The package to derive the new package from
    /// * `dist_dir` - The directory to base all dependency symlinks in `link/` on
    pub fn from_built(built_package: BuiltPackage, dist_dir: &Path) -> Result<Self, Error> {
        let context = || {
            format!(
                "Creating installable package {} from built package",
                built_package.get_full_name()
            )
        };

        // Assert the `dist_dir` is relative so it can be joined to other paths
        let dist_dir = assert_relative!(dist_dir).e_context(context)?;

        // Store the package path (<path>/data/<package>)
        let package_path = built_package
            .get_real_path()
            .join("data")
            .join(built_package.get_path(dist_dir));

        // Create a package metadata file struct
        let meta_file = package_meta::PackageMetaFile::from_package(&built_package);

        // Write it to the package root
        write_toml(&package_path.join("package.toml"), &meta_file).e_context(context)?;
        // Write it to the package archive root
        write_toml(
            &built_package.get_real_path().join("package.toml"),
            &meta_file,
        )
        .e_context(context)?;

        // Create the `link/` directory
        create_dir_all(&package_path.join("link")).e_context(context)?;

        // Populate the `link/` directory
        for dep in &built_package.dependencies {
            let dest = Path::new("/")
                .join(dist_dir)
                .join(&dep.arch)
                .join(&dep.name)
                .join(&dep.version);

            create_symlink(&package_path.join("link").join(&dep.name), &dest).e_context(context)?;
        }

        // Done, return a new installable package
        Ok(Self {
            name: built_package.name,
            version: built_package.version,
            arch: built_package.arch,
            description: built_package.description,
            dependencies: built_package.dependencies,
            path: built_package.path,
            index: built_package.index,
            build_id: built_package.build_id,
        })
    }
}

impl NamedPackage for InstallablePackage {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl VersionedPackage for InstallablePackage {
    fn get_version(&self) -> &str {
        &self.version
    }
}

impl ArchitecturePackage for InstallablePackage {
    fn get_arch(&self) -> &str {
        &self.arch
    }
}

impl NameVersionPackage for InstallablePackage {}

impl CorePackage for InstallablePackage {}

impl IndexedPackage for InstallablePackage {
    fn get_index(&self) -> &Directory {
        &self.index
    }
}

impl DescribedPackage for InstallablePackage {
    fn get_description(&self) -> &str {
        &self.description
    }
}

impl PathPackage for InstallablePackage {
    fn get_real_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl BuildIDProvider for InstallablePackage {
    fn get_build_id(&self) -> &str {
        &self.build_id
    }
}

impl DependencyProvider for InstallablePackage {
    fn get_dependencies(&self) -> Vec<&PackageInfo> {
        self.dependencies.iter().collect()
    }
}