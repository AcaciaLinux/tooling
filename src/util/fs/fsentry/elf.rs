use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use elf::{endian::AnyEndian, ElfBytes};

use crate::{
    error::{Error, ErrorExt},
    util::elf::ELFExt,
};

/// A structure representing an ELF file
#[derive(Clone)]
pub struct ELFFile {
    /// The type of ELF file at hand
    pub ty: ELFType,
    /// The entry point for the file, if there is none, this is `0`
    pub entry_point: u64,
    /// The interpreter requested by the binary (if available)
    pub interpreter: Option<PathBuf>,
    /// The needed shared libraries
    pub shared_needed: Vec<OsString>,
    /// The `RUNPATH` array of search paths for the dynamic linker
    pub runpaths: Vec<OsString>,

    /// The name of the file
    pub name: OsString,
}

/// The type of ELF file at hand
#[repr(u16)]
#[derive(Clone, PartialEq)]
pub enum ELFType {
    /// `ET_NONE`
    Unknown = 0,
    /// `ET_REL`
    Relocatable = 1,
    /// `ET_EXEC`
    Executable = 2,
    /// `ET_DYN`
    Shared = 3,
    /// `ET_CORE`
    Core = 4,
    /// Others
    Other = 0xFFFF,
}

impl ELFFile {
    /// Parses an `ELFFile` from the provided path
    /// # Arguments
    /// * `path` - The path to parse the file from
    /// * `name` - The name for the parsed file
    pub fn parse(path: &Path, name: OsString) -> Result<ELFFile, Error> {
        let file_data =
            std::fs::read(path).e_context(|| format!("Reading {}", &path.to_string_lossy()))?;
        let slice = file_data.as_slice();

        let file = ElfBytes::<AnyEndian>::minimal_parse(slice)
            .e_context(|| format!("Parsing ELF file at {}", &path.to_string_lossy()))?;

        let elf_file_struct = ELFFile {
            ty: ELFType::from_u16(file.ehdr.e_type),
            entry_point: file.ehdr.e_entry,
            interpreter: file.get_interpreter().e_context(|| {
                format!(
                    "Reading interpreter of ELF file {}",
                    &path.to_string_lossy()
                )
            })?,

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
                .map(|s| s.to_owned())
                .collect(),

            runpaths: file
                .get_runpaths()
                .e_context(|| format!("Reading RUNPATHs of ELF file {}", &path.to_string_lossy()))?
                .unwrap_or_default()
                .iter()
                .map(|s| s.to_owned())
                .collect(),

            name,
        };

        Ok(elf_file_struct)
    }

    /// Checks if this ELF file is an executable by
    /// - Checking if the `ty` is `Executable` or
    /// - The `ty` is `Shared` and the `entry_point` is **not** 0
    pub fn is_executable(&self) -> bool {
        self.ty == ELFType::Executable || (self.ty == ELFType::Shared && self.entry_point != 0)
    }

    /// Checks if this ELF file is a library by
    /// - Checking if the `ty` is `Shared`
    /// - And the `entry_point` **is** 0
    pub fn is_library(&self) -> bool {
        self.ty == ELFType::Shared && self.entry_point == 0
    }
}

impl ELFType {
    /// Parses the elf type from a `u16`. Invalid values result in `Self::Other`
    pub fn from_u16(v: u16) -> Self {
        match v {
            0 => Self::Unknown,
            1 => Self::Relocatable,
            2 => Self::Executable,
            3 => Self::Shared,
            4 => Self::Core,
            _ => Self::Other,
        }
    }
}
