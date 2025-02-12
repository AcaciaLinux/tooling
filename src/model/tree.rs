//! Data structures for representing and storing the AcaciaLinux index files

mod treecommand;
pub use treecommand::*;

use core::panic;
use log::{debug, trace};
use std::{
    io::{Cursor, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use crate::{
    error::{Error, ErrorExt},
    model::ObjectDB,
    util::{
        self,
        fs::{PathUtil, UNIXInfo},
        ODBUnpackable, Packable,
    },
};

use super::{Object, ObjectCompression, ObjectID, ObjectType};

/// The current version of the tree file
pub static CURRENT_VERSION: u8 = 0;

/// The representing structure for the index file
#[derive(Debug, PartialEq, Eq)]
pub struct Tree {
    /// The entries listed in the tree
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    /// Creates a new tree by recursively indexing `root` and creating subtrees along the way.
    /// # Arguments
    /// * `root` - The directory to index and insert
    /// * `db` - The object database to insert into
    /// * `compression` - The form of compression to use when inserting
    /// # Returns
    /// The indexed tree
    pub fn index(
        root: &Path,
        db: &mut ObjectDB,
        compression: ObjectCompression,
    ) -> Result<Tree, Error> {
        let mut entries: Vec<TreeEntry> = Vec::new();

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
                entries.push(TreeEntry::Symlink {
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
                let tree = Tree::index(&path, db, compression)?;
                entries.push(TreeEntry::Subtree {
                    info: unix_info,
                    name,
                    tree,
                });
            } else {
                // Files get hashed normally
                let object = db.insert_file_infer(&path, compression)?;
                entries.push(TreeEntry::File {
                    info: unix_info,
                    name,
                    oid: object.oid,
                });
            }
        }

        // Sort the entries alphabetically
        entries.sort();

        let tree = Tree { entries };

        Ok(tree)
    }

    /// Merges another tree into this tree by following
    /// these rules:
    /// - A non-existing (by name) entry gets added
    /// - Existing entries (by name) keep the name and UNIX info of the existing entry
    /// - Subtrees get merged in the same way
    /// # Arguments
    /// * `other` - The other tree to merge
    pub fn merge(&mut self, other: Tree) {
        for entry in other.entries {
            match self.get_entry_by_name_mut(entry.name()) {
                None => self.entries.push(entry),
                Some(my_entry) => {
                    if let TreeEntry::Subtree {
                        info: _,
                        name: _,
                        tree: my_tree,
                    } = my_entry
                    {
                        if let TreeEntry::Subtree {
                            info: _,
                            name: _,
                            tree,
                        } = entry
                        {
                            my_tree.merge(tree);
                        }
                    }
                }
            }
        }

        self.entries.sort();
    }

    /// Walks the index file and yields the entries
    /// # Arguments
    /// * `function` - The yield function providing the current working directory and the command to be executed
    pub fn walk<F: FnMut(&Path, &TreeEntry) -> Result<bool, Error>>(
        &self,
        function: &mut F,
        _odb: &ObjectDB,
    ) -> Result<(), Error> {
        let path = PathBuf::new();

        for command in &self.entries {
            if !function(&path, command)? {
                break;
            }

            if let TreeEntry::Subtree {
                info: _,
                name: _,
                tree,
            } = command
            {
                tree.walk(function, _odb)?;
            }
        }

        Ok(())
    }

    /// Returns the dependencies this tree uses with no recursion
    pub fn get_dependencies(&self) -> Vec<ObjectID> {
        let mut dependencies = Vec::new();

        for command in &self.entries {
            match command {
                TreeEntry::File {
                    info: _,
                    name: _,
                    oid,
                } => dependencies.push(oid.clone()),
                TreeEntry::Symlink {
                    info: _,
                    name: _,
                    destination: _,
                } => {}
                TreeEntry::Subtree {
                    info: _,
                    name: _,
                    tree,
                } => dependencies.push(tree.oid()),
            }
        }

        dependencies
    }

    /// Inserts `self` into the object database
    /// # Arguments
    /// * `db` - The object database to insert into
    /// * `compression` - The form of compression to use when inserting
    /// # Returns
    /// The inserted [Object]
    pub fn insert_into_odb(
        &self,
        db: &mut ObjectDB,
        compression: ObjectCompression,
    ) -> Result<Object, Error> {
        // Before inserting self, we must insert all subtrees
        for entry in &self.entries {
            if let TreeEntry::Subtree {
                info: _,
                name: _,
                tree,
            } = entry
            {
                tree.insert_into_odb(db, compression)?;
            }
        }

        let mut buf = Vec::new();
        self.pack(&mut buf)?;
        let mut buf = Cursor::new(buf);

        let object = db.insert_stream(
            &mut buf,
            ObjectType::AcaciaTree,
            compression,
            self.get_dependencies(),
        )?;

        debug!(
            "Inserting tree with {} children as {}",
            self.entries.len(),
            object.oid
        );

        Ok(object)
    }

    /// Deploys this index to `root`
    /// # Arguments
    /// * `root` - The root directory to deploy to
    /// * `db` - The object database to use for getting objects
    pub fn deploy(&self, root: &Path, db: &ObjectDB) -> Result<(), Error> {
        util::fs::create_dir_all(root).ctx(|| "Creating parent directory")?;

        for command in &self.entries {
            debug!("Executing {command} @ {}", root.str_lossy());
            command.execute(root, db)?;
        }

        Ok(())
    }

    /// Returns the object id derived from this tree
    pub fn oid(&self) -> ObjectID {
        let mut buf = Vec::new();
        self.pack(&mut buf)
            .expect("[DEV] Packing to a vec should never fail");
        let mut buf = Cursor::new(buf);

        ObjectID::new_from_stream(&mut buf, &self.get_dependencies())
            .expect("Hashing should never fail")
    }

    /// Returns a reference to an entry by name, if available
    /// # Arguments
    /// * `name` - The name of the entry
    pub fn get_entry_by_name(&self, name: &str) -> Option<&TreeEntry> {
        self.entries.iter().find(|entry| entry.name() == name)
    }

    /// Returns a mutable reference to an entry by name, if available
    /// # Arguments
    /// * `name` - The name of the entry
    pub fn get_entry_by_name_mut(&mut self, name: &str) -> Option<&mut TreeEntry> {
        self.entries.iter_mut().find(|entry| entry.name() == name)
    }
}

impl Packable for Tree {
    fn pack<W: Write>(&self, out: &mut W) -> Result<(), Error> {
        let context = || "Writing index file";

        out.write(b"ALTR").e_context(context)?;
        out.write(&[CURRENT_VERSION]).e_context(context)?;

        // When inserting, trees MUST be sorted
        if !self.entries.is_sorted() {
            panic!("[DEV] Tried to pack a non-sorted tree")
        }

        for entry in &self.entries {
            entry.pack(out)?;
        }

        Ok(())
    }
}

impl ODBUnpackable for Tree {
    fn try_unpack_from_odb<R: Read>(input: &mut R, odb: &ObjectDB) -> Result<Option<Self>, Error> {
        let context = || "Parsing index entry";

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

        let mut entries: Vec<TreeEntry> = Vec::new();

        while let Some(entry) = TreeEntry::try_unpack_from_odb(input, odb).ctx(context)? {
            trace!("Unpacked entry: {:x?}", entry);
            entries.push(entry)
        }

        Ok(Some(Tree { entries }))
    }
}
