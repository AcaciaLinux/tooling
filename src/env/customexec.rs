use std::{collections::HashMap, ffi::OsString};

use crate::env::EnvironmentExecutable;

/// An executable that can execute arbitrary programs
pub struct CustomExecutable {
    /// The program to execute
    pub program: String,
    /// The environment variables
    pub env_vars: HashMap<String, String>,
}

impl CustomExecutable {
    /// Creates a new custom executable, executing the supplied program
    /// # Arguments
    /// * `program` - The program to execute
    /// * `env_vars` - The environment variables to use
    pub fn new(program: String, env_vars: HashMap<String, String>) -> Self {
        Self { program, env_vars }
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

    fn get_workdir(&self) -> OsString {
        "/".into()
    }
}
