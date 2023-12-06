//! Environment structures to represent different environments for actions to take place

mod buildenv;
pub use buildenv::*;

use std::{collections::HashMap, ffi::OsString};

use crate::{error::Error, util::signal::SignalDispatcher};

mod customexec;
pub use customexec::*;

/// An environment that can execute `EnvironmentExecutables`
pub trait Environment {
    /// Executes a `EnvironmentExecutable` in the environment
    /// # Arguments
    /// * `executable` - A reference to the executable to execute
    /// * `signal_dispatcher` - A reference to the `SignalDispatcher` to register signals for the executed process
    fn execute(
        &self,
        executable: &dyn EnvironmentExecutable,
        signal_dispatcher: &SignalDispatcher,
    ) -> Result<std::process::ExitStatus, Error>;
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
