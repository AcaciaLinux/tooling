//! The indexer tool facilitates indexing directories and creating an object database from it

mod index;
pub use index::*;

use std::{collections::HashSet, path::PathBuf};

use crate::{
    error::Error,
    model::{ObjectCompression, ObjectDB, ObjectID, ObjectType},
    util::fs::{self, IndexCommand},
};

/// The index tool can index and hash filesystem trees
/// and prouce [Index] files
pub struct Indexer {
    /// The root to index
    root: PathBuf,
}

impl Indexer {
    /// Creates a new indexer tool
    ///
    /// This will not index anything yet
    /// # Arguments
    /// * `root` - The root this indexer indexes
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Runs the indexing operation, walking `root` and producing an [Index]
    /// # Arguments
    /// * `recursive` - Walk the filesystem tree recursively
    /// * `db` - The object database to put the indexed objects into
    /// * `compression` - The compression to use for insertion
    /// * `skip_duplicates` - Whether to skip already existing entries
    pub fn run(
        &self,
        recursive: bool,
        db: &mut ObjectDB,
        compression: ObjectCompression,
        skip_duplicates: bool,
    ) -> Result<Index, Error> {
        let mut index = Vec::new();
        let mut objects: HashSet<ObjectID> = HashSet::new();

        let mut path = PathBuf::from(&self.root);

        fs::walk_dir_commands(&self.root, recursive, &mut |mut command| {
            match &mut command {
                IndexCommand::DirectoryUP => {
                    path.pop();
                }
                IndexCommand::Directory { info: _, name } => path.push(name),
                IndexCommand::File {
                    info: _,
                    name,
                    ref mut oid,
                } => {
                    path.push(name);
                    let object = db.insert_file(
                        &path,
                        ObjectType::Other,
                        compression,
                        skip_duplicates,
                        Vec::new(),
                    )?;
                    *oid = object.oid;
                    objects.insert(oid.clone());
                    path.pop();
                }
                IndexCommand::Symlink {
                    info: _,
                    name: _,
                    dest: _,
                } => {}
            }

            index.push(command);
            Ok(true)
        })?;

        Ok(Index::new(index, objects))
    }
}
