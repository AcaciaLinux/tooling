use std::{
    fmt::Display,
    io::{self, ErrorKind, Read},
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

use super::Tree;

#[derive(Debug)]
pub enum TreeCommand {
    File {
        /// UNIX information about the file
        info: UNIXInfo,
        /// The name of the file
        name: String,
        /// The object ID to use for this file
        oid: ObjectID,
    },
    Symlink {
        /// UNIX information about the symlink
        info: UNIXInfo,
        /// The name of the symlink
        name: String,
        /// The destination the symlink points to
        destination: String,
    },
    Subtree {
        /// UNIX information about the subtree
        info: UNIXInfo,
        /// The name of the tree in the current directory
        name: String,
        /// The object ID of the tree to place
        oid: ObjectID,
    },
}
impl TreeCommand {
    /// Executes this index command in `path`
    /// # Arguments
    /// * `path` - The working directory to execute the command in
    /// * `db` - The object database to use for retrieving objects
    pub fn execute(&self, path: &Path, db: &ObjectDB) -> Result<(), Error> {
        match self {
            Self::File { info, name, oid } => {
                let path = path.join(name);
                let mut object = db.read(oid).ctx(|| "Retrieving object")?;

                let mut file =
                    fs::file_create(&path).ctx(|| format!("Creating file {}", path.str_lossy()))?;

                info.apply_file(&mut file)
                    .ctx(|| format!("Applying UNIX info to {}", path.str_lossy()))?;

                io::copy(&mut object, &mut file).ctx(|| "Copying data")?;
            }

            Self::Symlink {
                info,
                name,
                destination,
            } => {
                let path = path.join(name);
                fs::create_symlink(&path, &PathBuf::from(destination))?;

                info.apply_path(&path)
                    .e_context(|| format!("Applying UNIX info to {}", path.str_lossy()))?;
            }

            Self::Subtree { info, name, oid } => {
                let mut object = db.read(oid).ctx(|| "Retrieving object")?;

                let tree = Tree::try_unpack(&mut object).ctx(|| "Unpacking subtree")?;

                let path = path.join(name);
                fs::create_dir_all(&path)?;

                info.apply_path(&path)
                    .e_context(|| format!("Applying UNIX info to {}", path.str_lossy()))?;

                tree.deploy(&path, db)?;
            }
        }

        Ok(())
    }
}

impl Unpackable for TreeCommand {
    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self>, Error> {
        let context = || "Reading tree command";
        let ty = match u8::unpack(input).e_context(context)? {
            Some(ty) => ty,
            None => return Ok(None),
        };

        let context = || format!("Reading index command '{}'", ty);

        Ok(Some(match ty {
            0x5 => {
                let mut oid = [0u8; 32];
                input.read_exact(&mut oid).ctx(context)?;
                let oid = ObjectID::new(oid);

                let info = UNIXInfo::try_unpack(input).e_context(context)?;
                let name_len = u32::try_unpack(input).e_context(context)?;
                let mut buf = vec![0u8; name_len as usize];
                input.read_exact(&mut buf).e_context(context)?;
                let name = String::from_utf8(buf).e_context(context)?;
                TreeCommand::Subtree { info, name, oid }
            }

            0x1 => {
                let mut oid = [0u8; 32];
                input.read_exact(&mut oid).e_context(context)?;
                let oid = ObjectID::new(oid);

                let info = UNIXInfo::try_unpack(input).e_context(context)?;

                let name_len = u32::try_unpack(input).e_context(context)?;
                let mut buf = vec![0u8; name_len as usize];
                input.read_exact(&mut buf).e_context(context)?;
                let name = String::from_utf8(buf).e_context(context)?;

                TreeCommand::File { info, name, oid }
            }

            0x2 => {
                let info = UNIXInfo::try_unpack(input).e_context(context)?;

                let name_len = u32::try_unpack(input).e_context(context)?;
                let dest_len = u32::try_unpack(input).e_context(context)?;

                let mut name = vec![0u8; name_len as usize];
                input.read_exact(&mut name).e_context(context)?;
                let name = String::from_utf8(name).e_context(context)?;

                let mut destination = vec![0u8; dest_len as usize];
                input.read_exact(&mut destination).e_context(context)?;
                let destination = String::from_utf8(destination).e_context(context)?;
                TreeCommand::Symlink {
                    info,
                    name,
                    destination,
                }
            }
            _ => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Got unknown tree command {:x}", ty),
                ))
                .ctx(context);
            }
        }))
    }
}
impl Packable for TreeCommand {
    fn pack<W: std::io::Write>(&self, output: &mut W) -> Result<(), crate::error::Error> {
        let context = || format!("Writing index command {:?}", self);

        let ty: u8 = match self {
            Self::File {
                info: _,
                name: _,
                oid: _,
            } => 0x1u8,
            Self::Symlink {
                info: _,
                name: _,
                destination: _,
            } => 0x2u8,
            Self::Subtree {
                info: _,
                name: _,
                oid: _,
            } => 0x5u8,
        };
        output.write(&[ty]).e_context(context)?;

        match self {
            Self::File { info, name, oid } => {
                output.write(oid.bytes()).e_context(context)?;
                info.pack(output).e_context(context)?;
                (name.len() as u32).pack(output).e_context(context)?;
                output.write(name.as_bytes()).e_context(context)?;
            }

            Self::Symlink {
                info,
                name,
                destination,
            } => {
                info.pack(output).e_context(context)?;
                (name.len() as u32).pack(output).e_context(context)?;
                (destination.len() as u32).pack(output).e_context(context)?;
                output.write(name.as_bytes()).e_context(context)?;
                output.write(destination.as_bytes()).e_context(context)?;
            }
            Self::Subtree { info, name, oid } => {
                oid.pack(output).ctx(context)?;
                info.pack(output).ctx(context)?;
                (name.len() as u32).pack(output).e_context(context)?;
                output.write(name.as_bytes()).e_context(context)?;
            }
        }

        Ok(())
    }
}

impl Display for TreeCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File { info: _, name, oid } => write!(f, "FILE [{oid}] => {name}"),
            Self::Symlink {
                info: _,
                name,
                destination,
            } => write!(f, "LINK {name} => {destination}"),
            Self::Subtree { info: _, name, oid } => write!(f, "TREE [{oid}] => {name}"),
        }
    }
}
