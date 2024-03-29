use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::env::EnvironmentExecutable;

/// An executable that can execute arbitrary programs
pub struct CustomExecutable {
    /// The program to execute
    pub program: String,
    /// The working directory for this executable
    pub workdir: PathBuf,
    /// The environment variables
    pub env_vars: HashMap<String, String>,
}

impl CustomExecutable {
    /// Creates a new custom executable, executing the supplied program
    /// # Arguments
    /// * `program` - The program to execute
    /// * `workdir` - The working directory for the executable
    /// * `env_vars` - The environment variables to use
    pub fn new(program: String, workdir: PathBuf, env_vars: HashMap<String, String>) -> Self {
        Self {
            program,
            workdir,
            env_vars,
        }
    }
}

impl EnvironmentExecutable for CustomExecutable {
    fn get_env_variables(&self) -> HashMap<String, String> {
        self.env_vars.clone()
    }

    fn get_command(&self) -> OsString {
        self.program.clone().into()
    }

    fn get_name(&self) -> String {
        "Run shell".to_string()
    }

    fn get_workdir(&self) -> &Path {
        &self.workdir
    }
}
