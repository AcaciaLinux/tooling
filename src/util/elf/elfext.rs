//! Extra functionality for the `elf` crate

use std::{ffi::OsString, path::PathBuf};

use elf::{endian::EndianParse, ElfBytes, ParseError};

static D_TAG_NEEDED: i64 = 1;
static D_TAG_RPATH: i64 = 15;
static D_TAG_RUNPATH: i64 = 29;

/// Extended trait function for handling ELF files
pub trait ELFExt {
    /// Returns the interpreter requested by the ELF binary if one is needed, else `None`
    fn get_interpreter(&self) -> Result<Option<PathBuf>, ParseError>;
    /// Returns the needed shared libraries if available, else `None`
    fn get_shared_needed(&self) -> Result<Option<Vec<OsString>>, ParseError>;
    /// Returns the runpaths split apart if available, else `None`
    fn get_runpaths(&self) -> Result<Option<Vec<OsString>>, ParseError>;
}

impl<'data, T: EndianParse> ELFExt for ElfBytes<'data, T> {
    fn get_interpreter(&self) -> Result<Option<PathBuf>, ParseError> {
        let section = match self.section_header_by_name(".interp")? {
            Some(s) => s,
            None => return Ok(None),
        };

        match std::str::from_utf8(self.section_data(&section)?.0) {
            Ok(s) => Ok(Some(PathBuf::from(s.trim_end_matches(0 as char)))),
            Err(_) => Ok(None),
        }
    }

    fn get_shared_needed(&self) -> Result<Option<Vec<OsString>>, ParseError> {
        let common = self.find_common_data()?;

        let dynsyms = match common.dynsyms_strs {
            None => return Ok(None),
            Some(s) => s,
        };

        let section_dyn = match self.dynamic()? {
            None => return Ok(None),
            Some(s) => s,
        };

        let mut res: Vec<OsString> = Vec::new();

        for sym in section_dyn {
            if sym.d_tag == D_TAG_NEEDED {
                match dynsyms.get(sym.d_val() as usize) {
                    Err(_) => continue,
                    Ok(s) => res.push(OsString::from(s.trim_end_matches(0 as char))),
                };
            }
        }

        Ok(Some(res))
    }

    fn get_runpaths(&self) -> Result<Option<Vec<OsString>>, ParseError> {
        let common = self.find_common_data()?;

        let dynsyms = match common.dynsyms_strs {
            None => return Ok(None),
            Some(s) => s,
        };

        let section_dyn = match self.dynamic()? {
            None => return Ok(None),
            Some(s) => s,
        };

        let mut res: Vec<OsString> = Vec::new();

        for sym in section_dyn {
            if sym.d_tag == D_TAG_RUNPATH || sym.d_tag == D_TAG_RPATH {
                match dynsyms.get(sym.d_val() as usize) {
                    Err(_) => continue,
                    Ok(s) => s
                        .split(':')
                        .for_each(|p| res.push(OsString::from(p.trim_end_matches(0 as char)))),
                };
            }
        }

        Ok(Some(res))
    }
}
