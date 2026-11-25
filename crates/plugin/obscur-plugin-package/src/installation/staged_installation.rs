use std::io::{self, Write};
use std::path::PathBuf;

use super::generation_name::GenerationName;
use super::{InstallationStore, InstalledPackage};
use crate::validation::PackageError;

/// An unpublished generation that is removed on drop unless committed.
#[derive(Debug)]
pub struct StagedInstallation<'a> {
    pub(super) store: &'a InstallationStore,
    pub(super) generation: tempfile::TempDir,
    pub(super) candidate: InstalledPackage,
    pub(super) previous: Option<PathBuf>,
}

impl StagedInstallation<'_> {
    /// Returns the staged package for host migration and trial activation.
    #[must_use]
    pub const fn candidate(&self) -> &InstalledPackage {
        &self.candidate
    }

    /// Publishes the staged package after successful host checks.
    /// A concurrent publication causes this commit to fail without replacement.
    pub fn commit(self) -> Result<InstalledPackage, PackageError> {
        let _lock = self.store.lock()?;
        let id = &self.candidate.manifest().id;
        let current = self.store.current(id)?.map(|installed| installed.directory);
        if current != self.previous {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "installation changed since staging",
            )
            .into());
        }

        let generation = GenerationName::from_directory(self.generation.path())?;
        let mut marker = tempfile::NamedTempFile::new_in(&self.store.root)?;
        marker.write_all(generation.as_str().as_bytes())?;
        marker.as_file().sync_all()?;
        marker
            .persist(self.store.marker(id))
            .map_err(|error| error.error)?;
        let _ = self.generation.keep();
        Ok(self.candidate)
    }
}
