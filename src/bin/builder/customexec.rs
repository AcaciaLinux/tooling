use std::{collections::HashMap, ffi::OsString};
use tooling::env::EnvironmentExecutable;

/// The executable that can execute arbitrary commands
pub struct CustomExecutable {
    /// The program to execute
    pub program: String,
}

impl CustomExecutable {
    /// Creates a new custom executable, executing the supplied program
    pub fn new(program: String) -> Self {
        Self { program }
    }
}

impl EnvironmentExecutable for CustomExecutable {
    fn get_env_variables(&self) -> HashMap<String, String> {
        HashMap::new()
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
