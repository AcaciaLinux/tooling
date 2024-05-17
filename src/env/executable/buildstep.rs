use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    abs_dist_dir,
    env::EnvironmentExecutable,
    package::{info::PackageInfo, CorePackage, NamedPackage, VersionedPackage},
    util::architecture::Architecture,
};

/// A build step in the package build pipeline
pub struct BuildStep {
    /// The name for the build step
    pub name: String,
    /// Information about the package that is to be built
    pub pkg_info: PackageInfo,
    /// The architecture to build for
    pub arch: Architecture,
    /// The command to execute when building
    pub command: String,
    /// The working directory for the build process in the chroot
    pub workdir: PathBuf,
    /// The directory to install into in the chroot
    pub install_dir: PathBuf,
}

impl EnvironmentExecutable for BuildStep {
    fn get_name(&self) -> String {
        self.name.to_string()
    }

    fn get_env_variables(&self) -> std::collections::HashMap<String, String> {
        let mut map = HashMap::new();

        let pkg_root = self
            .pkg_info
            .get_root_dir(&abs_dist_dir())
            .to_string_lossy()
            .to_string();

        let install_dir = self.install_dir.to_string_lossy().to_string();

        map.insert("PKG_NAME", self.pkg_info.get_name());
        map.insert("PKG_VERSION", self.pkg_info.get_version());
        map.insert("PKG_ARCH", &self.arch.arch);
        map.insert("PKG_INSTALL_DIR", &install_dir);
        map.insert("PKG_ROOT", &pkg_root);

        map.into_iter()
            .map(|p| (p.0.to_string(), p.1.to_string()))
            .collect()
    }

    fn get_command(&self) -> std::ffi::OsString {
        self.command.clone().into()
    }

    fn get_workdir(&self) -> &Path {
        &self.workdir
    }
}
