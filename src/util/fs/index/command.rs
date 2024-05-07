use std::io::{ErrorKind, Read, Write};

use crate::{
    error::{Error, ErrorExt},
    model::ObjectID,
    util::{fs::UNIXInfo, Packable},
};

use super::IndexCommandType;

impl Packable for u8 {
    type Output = Self;

    fn pack<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        output
            .write(&[*self])
            .e_context(|| format!("Writing {self}"))?;

        Ok(())
    }

    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self::Output>, Error> {
        let mut buf = [0u8; 1];
        let x = input.read(&mut buf).e_context(|| "Read u8".to_owned())?;
        Ok(match x {
            1 => Some(buf[0]),
            _ => None,
        })
    }
}

impl Packable for u16 {
    type Output = Self;

    fn pack<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        output
            .write(&self.to_le_bytes())
            .e_context(|| format!("Writing {self}"))?;

        Ok(())
    }

    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self::Output>, Error> {
        let mut buf = [0u8; 2];
        let x = input.read(&mut buf).e_context(|| "Read u16".to_owned())?;
        Ok(match x {
            2 => Some(Self::from_le_bytes(buf)),
            _ => None,
        })
    }
}

impl Packable for u32 {
    type Output = Self;

    fn pack<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        output
            .write(&self.to_le_bytes())
            .e_context(|| format!("Writing {self}"))?;

        Ok(())
    }

    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self::Output>, Error> {
        let mut buf = [0u8; 4];
        let x = input.read(&mut buf).e_context(|| "Read u32".to_owned())?;
        Ok(match x {
            4 => Some(Self::from_le_bytes(buf)),
            _ => None,
        })
    }
}

/// Commands that describe how to walk a filesystem index
#[derive(Debug)]
#[repr(u8)]
pub enum IndexCommand {
    /// Go up one directory
    DirectoryUP,
    /// Create a directory (if not done already) and change into it
    Directory {
        info: UNIXInfo,
        /// The name of the directory
        name: String,
    },
    /// Place a file in the current directory named `name` using object `oid`
    File {
        info: UNIXInfo,
        /// The name of the file
        name: String,
        /// The object ID to use for this file
        oid: ObjectID,
    },
    /// Create a symlink in the current directory named `name` that points to `dest`
    Symlink {
        info: UNIXInfo,
        /// The name of the symlink
        name: String,
        /// The destination the symlink points to
        dest: String,
    },
}

impl Packable for IndexCommand {
    type Output = Self;

    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self>, Error> {
        let context = || "Reading index command type";
        let ty = match u8::unpack(input).e_context(context)? {
            Some(ty) => ty,
            None => return Ok(None),
        };

        let ty = match IndexCommandType::from_u8(ty) {
            Some(ty) => ty,
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Got unknown index command type {:x}", ty),
                ))
                .e_context(context);
            }
        };

        let context = || format!("Reading index command '{}'", ty.to_str());

        Ok(Some(match ty {
            IndexCommandType::DirectoryUP => IndexCommand::DirectoryUP,

            IndexCommandType::Directory => {
                let info = UNIXInfo::try_unpack(input).e_context(context)?;
                let name_len = u32::try_unpack(input).e_context(context)?;
                let mut buf = vec![0u8; name_len as usize];
                input.read_exact(&mut buf).e_context(context)?;
                let name = String::from_utf8(buf).e_context(context)?;
                IndexCommand::Directory { info, name }
            }

            IndexCommandType::File => {
                let info = UNIXInfo::try_unpack(input).e_context(context)?;

                let name_len = u32::try_unpack(input).e_context(context)?;
                let oid_len = u32::try_unpack(input).e_context(context)?;

                let mut buf = vec![0u8; name_len as usize];
                input.read_exact(&mut buf).e_context(context)?;
                let name = String::from_utf8(buf).e_context(context)?;

                let mut oid = vec![0u8; oid_len as usize];
                input.read_exact(&mut oid).e_context(context)?;
                let oid = ObjectID::new(oid);
                IndexCommand::File { info, name, oid }
            }

            IndexCommandType::Symlink => {
                let info = UNIXInfo::try_unpack(input).e_context(context)?;

                let name_len = u32::try_unpack(input).e_context(context)?;
                let dest_len = u32::try_unpack(input).e_context(context)?;

                let mut name = vec![0u8; name_len as usize];
                input.read_exact(&mut name).e_context(context)?;
                let name = String::from_utf8(name).e_context(context)?;

                let mut dest = vec![0u8; dest_len as usize];
                input.read_exact(&mut dest).e_context(context)?;
                let dest = String::from_utf8(dest).e_context(context)?;
                IndexCommand::Symlink { info, name, dest }
            }
        }))
    }

    fn pack<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        let context = || format!("Writing index command {:?}", self);

        let ty = IndexCommandType::from_command(self);
        output.write(&[ty as u8]).e_context(context)?;

        match self {
            Self::DirectoryUP => {}

            Self::Directory { info, name } => {
                info.pack(output).e_context(context)?;
                (name.len() as u32).pack(output).e_context(context)?;
                output.write(name.as_bytes()).e_context(context)?;
            }

            Self::File { info, name, oid } => {
                info.pack(output).e_context(context)?;
                (name.len() as u32).pack(output).e_context(context)?;
                (oid.len() as u32).pack(output).e_context(context)?;
                output.write(name.as_bytes()).e_context(context)?;
                output.write(oid.bytes()).e_context(context)?;
            }

            Self::Symlink { info, name, dest } => {
                info.pack(output).e_context(context)?;
                (name.len() as u32).pack(output).e_context(context)?;
                (dest.len() as u32).pack(output).e_context(context)?;
                output.write(name.as_bytes()).e_context(context)?;
                output.write(dest.as_bytes()).e_context(context)?;
            }
        }

        Ok(())
    }
}
