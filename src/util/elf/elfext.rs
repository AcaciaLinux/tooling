//! Extra functionality for the `elf` crate

use elf::{endian::EndianParse, ElfBytes, ParseError};

/// Extended trait function for handling ELF files
pub trait ELFExt {
    /// Returns the interpreter requested by the ELF binary if one is needed, else `None`
    fn get_interpreter(&self) -> Result<Option<&str>, ParseError>;
    /// Returns the needed shared libraries if available, else `None`
    fn get_shared_needed(&self) -> Result<Option<Vec<&str>>, ParseError>;
}

impl<'data, T: EndianParse> ELFExt for ElfBytes<'data, T> {
    fn get_interpreter(&self) -> Result<Option<&str>, ParseError> {
        let section = match self.section_header_by_name(".interp")? {
            Some(s) => s,
            None => return Ok(None),
        };

        match std::str::from_utf8(self.section_data(&section)?.0) {
            Ok(s) => Ok(Some(s)),
            Err(_) => Ok(None),
        }
    }

    fn get_shared_needed(&self) -> Result<Option<Vec<&str>>, ParseError> {
        let common = self.find_common_data()?;

        let dynsyms = match common.dynsyms_strs {
            None => return Ok(None),
            Some(s) => s,
        };

        let section_dyn = match self.dynamic()? {
            None => return Ok(None),
            Some(s) => s,
        };

        let mut res: Vec<&str> = Vec::new();

        for sym in section_dyn {
            if sym.d_tag != 1 {
                continue;
            }

            match dynsyms.get(sym.d_val() as usize) {
                Err(_) => continue,
                Ok(s) => res.push(s),
            };
        }

        Ok(Some(res))
    }
}
