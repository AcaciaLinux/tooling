//! Data structures for representing and storing the AcaciaLinux index files

pub mod treecommand;
pub use treecommand::*;

use std::{
    io::{Cursor, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use log::{debug, trace};

use crate::{
    error::{Error, ErrorExt},
    model::ObjectDB,
    util::{
        self,
        fs::{PathUtil, UNIXInfo},
        Packable, Unpackable,
    },
};

use super::{Object, ObjectCompression, ObjectID, ObjectType};

/// The current version of the tree file
pub static CURRENT_VERSION: u8 = 0;

/// The representing structure for the index file
#[derive(Debug)]
pub struct Tree {
    /// The commands listed in the file
    commands: Vec<TreeCommand>,
}

impl Tree {
    /// Creates a new tree by recursively indexing `root` and creating subtrees along the way.
    /// This also inserts the created tree and all objects into the object database.
    /// # Arguments
    /// * `root` - The directory to index and insert
    /// * `db` - The object database to insert into
    /// * `compression` - The form of compression to use when inserting
    /// * `skip_duplicates` - Whether to skip already existing entries or overwrite them
    /// # Returns
    /// The indexed tree and the matching [Object]
    pub fn index(
        root: &Path,
        db: &mut ObjectDB,
        compression: ObjectCompression,
        skip_duplicates: bool,
    ) -> Result<(Tree, Object), Error> {
        let mut commands: Vec<TreeCommand> = Vec::new();
        for entry in std::fs::read_dir(root).ctx(|| format!("Walking {}", root.str_lossy()))? {
            let entry = entry.ctx(|| "Reading filesystem entry")?;
            let unix_info = UNIXInfo::from_entry(&entry).ctx(|| "Getting UNIX info")?;
            let name = entry
                .path()
                .file_name()
                .expect("[BUG] Files MUST have a name?")
                .to_string_lossy()
                .to_string();

            let path = root.join(&name);

            if path.is_symlink() {
                // We first check for symlinks, as all other functions follow symlinks
                commands.push(TreeCommand::Symlink {
                    info: unix_info,
                    name,
                    destination: path
                        .read_link()
                        .ctx(|| "Reading link target")?
                        .to_string_lossy()
                        .to_string(),
                })
            } else if path.is_dir() {
                // Directories get linked to as subtrees
                let (_, object) = Tree::index(&path, db, compression, skip_duplicates)?;
                commands.push(TreeCommand::Subtree {
                    info: unix_info,
                    name,
                    oid: object.oid,
                });
            } else {
                // Files get hashed normally
                let object = db.insert_file_infer(&path, compression, skip_duplicates)?;
                commands.push(TreeCommand::File {
                    info: unix_info,
                    name,
                    oid: object.oid,
                });
            }
        }

        let tree = Tree { commands };

        let object = tree
            .insert(db, compression, skip_duplicates)
            .ctx(|| "Inserting this tree")?;

        Ok((tree, object))
    }

    /// Walks the index file and yields the entries
    /// # Arguments
    /// * `function` - The yield function providing the current working directory and the command to be executed
    pub fn walk<F: FnMut(&Path, &TreeCommand) -> Result<bool, Error>>(
        &self,
        function: &mut F,
        odb: &ObjectDB,
    ) -> Result<(), Error> {
        let path = PathBuf::new();

        for command in &self.commands {
            if !function(&path, command)? {
                break;
            }

            if let TreeCommand::Subtree {
                info: _,
                name: _,
                oid,
            } = command
            {
                let mut obj = odb.read(oid)?;
                let tree = Tree::try_unpack(&mut obj)?;

                tree.walk(function, odb)?;
            }
        }

        Ok(())
    }

    /// Returns the dependencies this tree uses with no recursion
    pub fn get_dependencies(&self) -> Vec<ObjectID> {
        let mut dependencies = Vec::new();

        for command in &self.commands {
            match command {
                TreeCommand::File {
                    info: _,
                    name: _,
                    oid,
                } => dependencies.push(oid.clone()),
                TreeCommand::Symlink {
                    info: _,
                    name: _,
                    destination: _,
                } => {}
                TreeCommand::Subtree {
                    info: _,
                    name: _,
                    oid,
                } => dependencies.push(oid.clone()),
            }
        }

        dependencies
    }

    /// Inserts `self` into the object database
    /// # Arguments
    /// * `db` - The object database to insert into
    /// * `compression` - The form of compression to use when inserting
    /// * `skip_duplicates` - Whether to skip already existing entries or overwrite them
    /// # Returns
    /// The inserted [Object]
    pub fn insert(
        &self,
        db: &mut ObjectDB,
        compression: ObjectCompression,
        skip_duplicates: bool,
    ) -> Result<Object, Error> {
        let mut buf = Vec::new();
        self.pack(&mut buf)?;
        let mut buf = Cursor::new(buf);

        db.insert_stream(
            &mut buf,
            ObjectType::AcaciaTree,
            compression,
            skip_duplicates,
            self.get_dependencies(),
        )
    }

    /// Deploys this index to `root`
    /// # Arguments
    /// * `root` - The root directory to deploy to
    /// * `db` - The object database to use for getting objects
    pub fn deploy(&self, root: &Path, db: &ObjectDB) -> Result<(), Error> {
        util::fs::create_dir_all(root).ctx(|| "Creating parent directory")?;

        for command in &self.commands {
            debug!("Executing {command} @ {}", root.str_lossy());
            command.execute(root, db)?;
        }

        Ok(())
    }
}

impl Packable for Tree {
    fn pack<W: Write>(&self, out: &mut W) -> Result<(), Error> {
        let context = || "Writing index file";

        out.write(b"ALTR").e_context(context)?;
        out.write(&[CURRENT_VERSION]).e_context(context)?;

        for command in &self.commands {
            command.pack(out)?;
        }

        Ok(())
    }
}

impl Unpackable for Tree {
    fn unpack<R: Read>(input: &mut R) -> Result<Option<Self>, Error> {
        let context = || "Parsing index command";

        let mut buf = [0u8; 4];
        input.read_exact(&mut buf).e_context(context)?;

        if buf != ['A', 'L', 'T', 'R'].map(|p| p as u8) {
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

        let mut commands: Vec<TreeCommand> = Vec::new();

        loop {
            let command = match TreeCommand::unpack(input).e_context(context)? {
                Some(c) => c,
                None => break,
            };

            trace!("Unpacked entry: {:x?}", command);
            commands.push(command);
        }

        Ok(Some(Tree { commands }))
    }
}
