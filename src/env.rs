//! Environment structures to represent different environments for actions to take place

mod buildenv;
use std::{collections::HashMap, ffi::OsString};

pub use buildenv::*;

/// An environment that can execute `EnvironmentExecutables`
pub trait Environment {
    /// Executes a `EnvironmentExecutable` in the environment
    /// # Arguments
    /// * `executable` - A reference to the executable to execute
    fn execute(
        &self,
        executable: &dyn EnvironmentExecutable,
    ) -> Result<std::process::ExitStatus, std::io::Error>;
}

/// An executable that can be executed in a `Environment`
pub trait EnvironmentExecutable {
    /// Returns the name of the executable to ease identification
    fn get_name(&self) -> String;

    /// Returns a hash map of the environment variables to pass to the environment
    fn get_env_variables(&self) -> HashMap<String, String>;

    /// Returns the command to execute in the environment
    fn get_command(&self) -> OsString;

    /// Returns the directory to run the command in
    fn get_workdir(&self) -> OsString;
}
