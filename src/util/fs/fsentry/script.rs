use std::{
    collections::LinkedList,
    ffi::OsString,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::error::{Error, ErrorExt};

/// A fs entry that is a script (has a shbang at its start)
#[derive(Clone)]
pub struct ScriptFile {
    /// The name of the file
    pub name: OsString,

    /// The interpreter for the file
    /// (`<path to the interpreter>`, `<array of arguments>`)
    pub interpreter: Option<(PathBuf, Vec<OsString>)>,
}

impl ScriptFile {
    /// Parses an `ELFFile` from the provided path
    /// # Arguments
    /// * `path` - The path to parse the file from
    pub fn parse(path: &Path, name: OsString) -> Result<Self, Error> {
        let context = || format!("Parsing SCRIPT file {}", path.to_string_lossy());

        // Read the first line
        let first_line = {
            let mut file = BufReader::new(File::open(path).e_context(context)?);
            let mut shbang = String::new();
            file.read_line(&mut shbang).e_context(context)?;
            shbang
        };

        // Remove the shbang from the start ('#!')
        let first_line = first_line.trim_start_matches("#!");

        // Split the line into its pieces
        let mut split: LinkedList<OsString> = first_line
            .split(' ')
            .map(|s| OsString::from(s.trim()))
            .collect();

        // Split the interpreter path from its arguments
        let interpreter = split.pop_front().map(|i| {
            (
                PathBuf::from(i),
                split.into_iter().collect::<Vec<OsString>>(),
            )
        });

        Ok(Self { name, interpreter })
    }
}
