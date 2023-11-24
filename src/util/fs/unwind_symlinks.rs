use std::path::{Path, PathBuf};

use log::trace;

/// Unwinds all symlinks to reach the real destination of the symlink for reading
/// from the linked file.
///
/// This function works recursively, following all further symlinks down the road
/// # Arguments
/// * `path` - The path to unwind
/// # Returns
/// The real destination of the symlink. If reading the link fails, the original path
pub fn unwind_symlinks(path: &Path) -> PathBuf {
    if path.is_symlink() {
        let mut new_path = path.to_path_buf();
        new_path.pop();
        let dest = match path.read_link() {
            Ok(l) => l,
            Err(e) => {
                trace!(
                    "Failed to read symlink destination of {}: {e}",
                    path.to_string_lossy()
                );
                return path.to_owned();
            }
        };
        new_path.push(dest);

        trace!(
            "Symlink '{}' points to '{}'",
            path.to_string_lossy(),
            new_path.to_string_lossy()
        );

        unwind_symlinks(&new_path)
    } else {
        path.to_path_buf()
    }
}
