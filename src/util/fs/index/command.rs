use std::{
    fmt::Display,
    io::{self, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use crate::{
    error::{Error, ErrorExt},
    model::{ObjectDB, ObjectID},
    util::{
        fs::{self, PathUtil, UNIXInfo},
        Packable, Unpackable,
    },
};

use super::IndexCommandType;

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

impl IndexCommand {
    /// Executes this index command in `path`
    /// # Arguments
    /// * `path` - The working directory to execute the command in
    /// * `db` - The object database to use for retrieving objects
    pub fn execute(&self, path: &Path, db: &ObjectDB) -> Result<(), Error> {
        match self {
            Self::File { info, name, oid } => {
                let path = path.join(name);
                let mut object = db.read(oid).e_context(|| "Retrieving object")?;

                let mut file = fs::file_create(&path)
                    .e_context(|| format!("Creating file {}", path.str_lossy()))?;

                info.apply_file(&mut file)
                    .e_context(|| format!("Applying UNIX info to {}", path.str_lossy()))?;

                io::copy(&mut object, &mut file).e_context(|| "Copying data")?;
            }

            Self::Directory { info, name } => {
                let path = path.join(name);
                fs::create_dir_all(&path)?;

                info.apply_path(&path)
                    .e_context(|| format!("Applying UNIX info to {}", path.str_lossy()))?;
            }

            Self::Symlink { info, name, dest } => {
                let path = path.join(name);
                fs::create_symlink(&path, &PathBuf::from(dest))?;

                info.apply_path(&path)
                    .e_context(|| format!("Applying UNIX info to {}", path.str_lossy()))?;
            }

            Self::DirectoryUP => {}
        }

        Ok(())
    }
}

impl Packable for IndexCommand {
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

impl Unpackable for IndexCommand {
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

                let mut buf = vec![0u8; name_len as usize];
                input.read_exact(&mut buf).e_context(context)?;
                let name = String::from_utf8(buf).e_context(context)?;

                let mut oid = [0u8; 32];
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
}

impl Display for IndexCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexCommand::DirectoryUP => write!(f, "CD .."),
            IndexCommand::Directory { info: _, name } => write!(f, "CD {name}"),
            IndexCommand::File { info: _, name, oid } => write!(f, "FILE {oid} => {name}"),
            IndexCommand::Symlink {
                info: _,
                name,
                dest,
            } => write!(f, "SYM {name} => {dest}"),
        }
    }
}
