use std::{collections::HashMap, path::PathBuf};

use crate::{dist_dir, env::EnvironmentExecutable, util::fs::PathUtil};

use super::Formula;

/// A buildstep specified in a formula source
pub struct FormulaBuildstep<'a> {
    /// A human name for that build step
    pub name: String,
    /// The command to execute
    pub command: String,
    /// The formula as a back-reference
    pub formula: &'a Formula,
    /// The working directory to work in
    pub workdir: PathBuf,
    /// The path to populate `PKG_INSTALL_DIR` with
    pub install_dir: PathBuf,
}

impl<'a> EnvironmentExecutable for FormulaBuildstep<'a> {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_env_variables(&self) -> std::collections::HashMap<String, String> {
        let mut map = HashMap::new();

        let oid_str = self.formula.oid().to_string();

        let pkg_root_str = dist_dir().join(PathBuf::from(oid_str)).str_lossy();
        let install_dir_str = self.install_dir.str_lossy();

        map.insert("PKG_NAME", &self.formula.name);
        map.insert("PKG_VERSION", &self.formula.version);
        if let Some(arch) = &self.formula.arch {
            map.insert("PKG_ARCH", &arch.arch);
        }
        map.insert("PKG_ROOT", &pkg_root_str);
        map.insert("PKG_INSTALL_DIR", &install_dir_str);

        map.into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn get_command(&self) -> std::ffi::OsString {
        self.command.clone().into()
    }

    fn get_workdir(&self) -> &std::path::Path {
        &self.workdir
    }
}

impl Formula {
    /// Returns the buildsteps specified by this formula
    /// # Arguments
    /// * `workdir` - The working directory for the buildsteps to run in
    /// * `install_dir` - The path to populate `PKG_INSTALL_DIR` with
    pub fn get_buildsteps(&self, workdir: PathBuf, install_dir: PathBuf) -> Vec<FormulaBuildstep> {
        let mut steps = Vec::new();

        if let Some(cmd) = &self.prepare {
            steps.push(self.create_buildstep("prepare", cmd, workdir.clone(), install_dir.clone()));
        }

        if let Some(cmd) = &self.build {
            steps.push(self.create_buildstep("build", cmd, workdir.clone(), install_dir.clone()));
        }

        if let Some(cmd) = &self.check {
            steps.push(self.create_buildstep("check", cmd, workdir.clone(), install_dir.clone()));
        }

        if let Some(cmd) = &self.package {
            steps.push(self.create_buildstep("package", cmd, workdir.clone(), install_dir.clone()));
        }

        steps
    }

    fn create_buildstep(
        &self,
        name: &str,
        command: &str,
        workdir: PathBuf,
        install_dir: PathBuf,
    ) -> FormulaBuildstep {
        FormulaBuildstep {
            name: name.to_string(),
            command: command.to_string(),
            formula: self,
            workdir,
            install_dir,
        }
    }
}
