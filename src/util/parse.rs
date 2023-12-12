//! Utilities for parsing files

use std::{fs::File, io::Write, path::Path};

use serde::{de::DeserializeOwned, Serialize};

use crate::error::{Error, ErrorExt};

pub mod versionstring;

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

/// Writes the contents of `string` to a file
/// # Arguments
/// * `path` - The path to write to
/// * `string` - The string to write
pub fn write_string_to_file(path: &Path, string: &str) -> Result<(), Error> {
    let context = || format!("Writing to file {}", path.to_string_lossy());

    let mut file = File::create(path).e_context(context)?;

    file.write_all(string.as_bytes()).e_context(context)?;

    Ok(())
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

/// Writes a serializable value to a toml file
/// # Arguments
/// * `path` - The path to write to
/// * `value` - The struct to serialize
pub fn write_toml<T>(path: &Path, value: &T) -> Result<(), Error>
where
    T: Serialize + ?Sized,
{
    let context = || format!("Writing TOML file {}", path.to_string_lossy());

    let string = toml::ser::to_string(value).unwrap();
    write_string_to_file(path, &string).e_context(context)?;

    Ok(())
}
