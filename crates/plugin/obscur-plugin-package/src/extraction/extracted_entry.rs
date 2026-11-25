//! Filesystem location of a verified payload file.

use std::path::{Path, PathBuf};

/// Filesystem location of a verified payload file.
#[derive(Debug)]
pub struct ExtractedEntry {
    pub(super) path: PathBuf,
}

impl ExtractedEntry {
    /// Returns the absolute path within the isolated directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}
