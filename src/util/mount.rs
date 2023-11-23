//! Utility functions for mounting filesystems

use std::path::Path;

mod overlay;
pub use overlay::*;

mod vkfs;
pub use vkfs::*;

mod bind;
pub use bind::*;

/// A common trait for all mount types
pub trait Mount {
    /// Returns a description of the type (`overlayfs`, `vkfs`...)
    fn get_fs_type(&self) -> String;
    /// Returns the path where the mount points to
    fn get_target_path(&self) -> &Path;
    /// Returns a / the first source path (or string)
    fn get_source_path(&self) -> &Path;
    /// Returns all source paths (overlayfs)
    fn get_source_paths(&self) -> Vec<&Path>;
}
