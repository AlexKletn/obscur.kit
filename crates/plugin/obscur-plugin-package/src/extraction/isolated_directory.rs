//! Owns an extracted payload directory and attempts cleanup on drop.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use super::ExtractedEntry;
use crate::path::PackagePath;

/// Owns an extracted payload directory and attempts cleanup on drop.
#[derive(Debug)]
pub struct IsolatedDirectory {
    pub(super) root: tempfile::TempDir,
    pub(super) files: BTreeMap<PackagePath, ExtractedEntry>,
    pub(super) entrypoint: PackagePath,
}

impl IsolatedDirectory {
    /// Returns the absolute path of the private extraction directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.root.path()
    }

    /// Returns the extracted file selected by the validated manifest.
    #[must_use]
    pub fn entrypoint(&self) -> &ExtractedEntry {
        &self.files[&self.entrypoint]
    }

    /// Finds a verified file by its normalized package path.
    #[must_use]
    pub fn file(&self, path: &PackagePath) -> Option<&ExtractedEntry> {
        self.files.get(path)
    }

    #[cfg(any(unix, windows))]
    pub(crate) fn move_to(self, path: &Path) -> io::Result<()> {
        std::fs::rename(self.root.path(), path)?;
        let _ = self.root.keep();
        Ok(())
    }

    /// Removes the directory immediately and reports a cleanup failure.
    pub fn close(self) -> io::Result<()> {
        self.root.close()
    }
}
