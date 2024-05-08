use std::path::{Path, PathBuf};

/// Common utility functions for `Path` structures
pub trait PathUtil {
    /// Makes the path relative by removing leading `/` characters
    fn make_relative(&self) -> &Path;
    /// Returns the path as a lossy string by using `.to_string_lossy()` and `.to_string()`
    fn str_lossy(&self) -> String;
}

impl PathUtil for PathBuf {
    fn make_relative(&self) -> &Path {
        match self.strip_prefix("/") {
            Ok(p) => p,
            Err(_) => self,
        }
    }
    fn str_lossy(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

impl PathUtil for Path {
    fn make_relative(&self) -> &Path {
        match self.strip_prefix("/") {
            Ok(p) => p,
            Err(_) => self,
        }
    }
    fn str_lossy(&self) -> String {
        self.to_string_lossy().to_string()
    }
}
