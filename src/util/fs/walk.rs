use std::path::Path;

/// Walks a directory, calling the callback for every entry found on the way.
/// # Arguments
/// * `path` - The path to walk
/// * `recursive` - If this function should operate recursively
/// * `callback` - The callback called for every entry, if it returns `false`, this function will stop
/// # Errors
/// Uses the std::fs::read_dir() function which will error on:
/// - The `path` does not exist
/// - Permission is denied
/// - The `path` is not a directory
pub fn walk_dir<F>(path: &Path, recursive: bool, callback: &mut F) -> Result<(), std::io::Error>
where
    F: FnMut(std::fs::DirEntry) -> bool,
{
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if !callback(entry) {
            return Ok(());
        }

        // Do only walk a subdirectory if it is not a symlink
        if !path.is_symlink() && path.is_dir() && recursive {
            walk_dir(&path, recursive, callback)?;
        }
    }

    Ok(())
}
