//! Data structures for representing and storing the AcaciaLinux index files

use std::{
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use log::debug;

use crate::{
    error::{Error, ErrorExt},
    util::{fs::IndexCommand, Packable, Unpackable},
};

/// The current version of the index file
pub static CURRENT_VERSION: u8 = 0;

/// The representing structure for the index file
#[derive(Debug)]
pub struct IndexFile {
    /// The version of the file (currently only `0`)
    pub version: u8,
    /// The commands listed in the file
    pub commands: Vec<IndexCommand>,
}

impl IndexFile {
    /// Walks the index file and yields the entries
    /// # Arguments
    /// * `function` - The yield function providing the current working directory and the command to be executed
    pub fn walk<F: FnMut(&Path, &IndexCommand) -> Result<bool, Error>>(
        &self,
        mut function: F,
    ) -> Result<(), Error> {
        let mut path = PathBuf::new();

        for command in &self.commands {
            if !function(&path, command)? {
                break;
            }

            match command {
                IndexCommand::DirectoryUP => {
                    path.pop();
                }
                IndexCommand::Directory { info: _, name } => path.push(name),
                _ => {}
            };
        }

        Ok(())
    }
}

impl Packable for IndexFile {
    fn pack<W: Write>(&self, out: &mut W) -> Result<(), Error> {
        let context = || "Writing index file";

        out.write(b"AIDX").e_context(context)?;
        out.write(&[CURRENT_VERSION]).e_context(context)?;

        for command in &self.commands {
            command.pack(out)?;
        }

        Ok(())
    }
}

impl Unpackable for IndexFile {
    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self>, Error> {
        let context = || "Parsing index command";

        let mut buf = [0u8; 4];
        input.read_exact(&mut buf).e_context(context)?;

        if buf != ['A', 'I', 'D', 'X'].map(|p| p as u8) {
            Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Expected file magic",
            ))
            .e_context(context)?;
        }

        let mut buf = [0u8];

        input.read_exact(&mut buf).e_context(context)?;
        if buf[0] != CURRENT_VERSION {
            Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Expected version to be {:x}, got {:x}",
                    CURRENT_VERSION, buf[0]
                ),
            ))
            .e_context(context)?;
        }

        let mut commands: Vec<IndexCommand> = Vec::new();

        loop {
            let command = match IndexCommand::unpack(input).e_context(context)? {
                Some(c) => c,
                None => break,
            };

            debug!("Got one entry: {:x?}", command);
            commands.push(command);
        }

        Ok(Some(IndexFile {
            version: 1,
            commands,
        }))
    }
}
