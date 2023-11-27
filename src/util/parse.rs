//! Utilities for parsing files

use std::path::Path;

use serde::de::DeserializeOwned;

use crate::error::{Error, ErrorExt};

/// Reads the contents of a file to a string
/// # Arguments
/// * `path` - The path to the file to read
/// # Returns
/// The string or an error
/// # Errors
/// Uses the `std::fs::read_to_string()` function, refer to it for errors
pub fn read_file_to_string(path: &Path) -> Result<String, Error> {
    let file_str = std::fs::read_to_string(path)
        .e_context(|| format!("Reading file {}", path.to_string_lossy()))?;
    Ok(file_str)
}

/// Parses the contents of the passed path, expecting a TOML file
/// # Arguments
/// * `path` - The path to the file to parse
/// # Returns
/// The parsed structure expected by the generic argument or an error
/// # Errors
/// Uses the `read_file_to_string()` function, refer to it for errors
pub fn parse_toml<T: DeserializeOwned>(path: &Path) -> Result<T, Error> {
    let context = || format!("Parsing TOML file {}", path.to_string_lossy());

    let file_str = read_file_to_string(path).e_context(context)?;
    let toml_content: T = toml::from_str(&file_str).e_context(context)?;

    Ok(toml_content)
}
