use std::collections::HashSet;

use crate::{
    files::index::{self, IndexFile},
    model::ObjectID,
    util::fs::IndexCommand,
};

/// An index that contains a list of objects and instructions on where to place them
/// to be able to reconstruct the filesystem somewhere else
pub struct Index {
    commands: Vec<IndexCommand>,
    objects: HashSet<ObjectID>,
}

impl Index {
    /// Creates a new index from the supplied information
    /// # Arguments
    /// * `commands` - The commands for reconstructing the filesystem tree
    /// * `objects` - The objects needed by this index
    pub fn new(commands: Vec<IndexCommand>, objects: HashSet<ObjectID>) -> Self {
        Self { commands, objects }
    }

    /// Creates a [IndexFile](crate::files::index::IndexFile) from this index
    pub fn to_index_file(self) -> index::IndexFile {
        IndexFile {
            version: index::CURRENT_VERSION,
            commands: self.commands,
        }
    }

    /// Returns the commands to reconstruct the index
    pub fn get_commands(&self) -> &Vec<IndexCommand> {
        &self.commands
    }

    /// Returns the objects needed for by this index
    pub fn get_objects(&self) -> &HashSet<ObjectID> {
        &self.objects
    }
}
