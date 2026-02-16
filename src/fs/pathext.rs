use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

/// Trait for adding extended functionality to path types like [Path] and [PathBuf]
pub trait PathExt {
    /// A shorthand for [Path::to_string_lossy()]
    fn str_lossy(&self) -> Cow<'_, str>;
}

impl PathExt for Path {
    fn str_lossy(&self) -> Cow<'_, str> {
        self.to_string_lossy()
    }
}

impl PathExt for PathBuf {
    fn str_lossy(&self) -> Cow<'_, str> {
        self.to_string_lossy()
    }
}
