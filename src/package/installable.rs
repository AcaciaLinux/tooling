use std::path::{Path, PathBuf};

use crate::{
    assert_relative,
    error::{Error, ErrorExt},
    files::package_meta,
    util::{
        fs::{create_dir_all, create_symlink, file_create, Directory},
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
    pub pkgver: u32,
    pub arch: String,
    pub description: String,

    pub dependencies: Vec<PackageInfo>,

    /// A list of directories in this package that contain executables
    pub executable_dirs: Vec<PathBuf>,
    /// A list of directories in this package that contain libraries
    pub library_dirs: Vec<PathBuf>,

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
            let dest = dep.get_path(&Path::new("/").join(dist_dir));

            create_symlink(&package_path.join("link").join(&dep.name), &dest).e_context(context)?;
        }

        // Done, return a new installable package
        Ok(Self {
            name: built_package.name,
            version: built_package.version,
            pkgver: built_package.pkgver,
            arch: built_package.arch,
            description: built_package.description,
            dependencies: built_package.dependencies,
            executable_dirs: built_package.executable_dirs,
            library_dirs: built_package.library_dirs,
            path: built_package.path,
            index: built_package.index,
            build_id: built_package.build_id,
        })
    }

    /// Archives this package into a `tar xz` archive file
    /// # Arguments
    /// * `archive_path` - The path to place the archived package at
    pub fn archive(&self, archive_path: &Path) -> Result<(), Error> {
        let context = || {
            format!(
                "Archiving package {} to '{}'",
                self.get_full_name(),
                archive_path.to_string_lossy()
            )
        };

        let xz_file = file_create(archive_path).e_context(context)?;
        let xz = xz::write::XzEncoder::new(xz_file, 9);

        let mut builder = tar::Builder::new(xz);
        builder.follow_symlinks(false);
        builder.mode(tar::HeaderMode::Deterministic);

        builder
            .append_dir_all(self.get_full_name(), self.get_real_path())
            .e_context(|| "When archiving".to_owned())
            .e_context(context)?;

        Ok(())
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
    fn get_pkgver(&self) -> u32 {
        self.pkgver
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

    fn get_executable_dirs(&self) -> &[PathBuf] {
        &self.executable_dirs
    }

    fn get_library_dirs(&self) -> &[PathBuf] {
        &self.library_dirs
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
