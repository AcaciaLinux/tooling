use std::{ffi::OsString, path::Path};

use elf::{endian::AnyEndian, ElfBytes};

use crate::{
    error::{Error, ErrorExt},
    util::elf::ELFExt,
};

/// A structure representing an ELF file
pub struct ELFFile {
    /// The interpreter requested by the binary (if available)
    pub interpreter: Option<String>,
    /// The needed shared libraries
    pub shared_needed: Vec<String>,
    /// The `RUNPATH` array of search paths for the dynamic linker
    pub runpaths: Vec<String>,

    /// The name of the file
    pub name: OsString,
}

impl ELFFile {
    /// Parses an `ELFFile` from the provided path
    /// # Arguments
    /// * `path` - The path to parse the file from
    pub fn parse(path: &Path) -> Result<ELFFile, Error> {
        let file_data =
            std::fs::read(path).e_context(|| format!("Reading {}", &path.to_string_lossy()))?;
        let slice = file_data.as_slice();

        let file = ElfBytes::<AnyEndian>::minimal_parse(slice)
            .e_context(|| format!("Parsing ELF file at {}", &path.to_string_lossy()))?;

        let elf_file_struct = ELFFile {
            interpreter: file
                .get_interpreter()
                .e_context(|| {
                    format!(
                        "Reading interpreter of ELF file {}",
                        &path.to_string_lossy()
                    )
                })?
                .map(|i| i.to_string()),

            shared_needed: file
                .get_shared_needed()
                .e_context(|| {
                    format!(
                        "Reading shared libraries of ELF file {}",
                        &path.to_string_lossy()
                    )
                })?
                .unwrap_or_default()
                .iter()
                .map(|s| s.to_string())
                .collect(),

            runpaths: file
                .get_runpaths()
                .e_context(|| format!("Reading RUNPATHs of ELF file {}", &path.to_string_lossy()))?
                .unwrap_or_default()
                .iter()
                .map(|s| s.to_string())
                .collect(),

            name: path.file_name().expect("Filename").to_owned(),
        };

        Ok(elf_file_struct)
    }
}
