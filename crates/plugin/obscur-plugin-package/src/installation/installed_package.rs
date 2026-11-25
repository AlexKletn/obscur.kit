use std::path::{Path, PathBuf};

use obscur_plugin_model::manifest::PluginManifest;

use crate::path::PackagePath;
use crate::validation::PackageError;

/// A package generation in the installation store.
#[derive(Debug)]
pub struct InstalledPackage {
    pub(super) directory: PathBuf,
    manifest: PluginManifest,
    entrypoint: PackagePath,
}

impl InstalledPackage {
    pub(super) fn new(directory: PathBuf, manifest: PluginManifest) -> Result<Self, PackageError> {
        let entrypoint = PackagePath::parse(&manifest.entrypoint.path)
            .ok_or_else(|| PackageError::InvalidPath(manifest.entrypoint.path.to_string()))?;
        Ok(Self {
            directory,
            manifest,
            entrypoint,
        })
    }

    /// Returns the manifest from the validated package.
    #[must_use]
    pub const fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Returns the directory containing the verified payload.
    #[must_use]
    pub fn payload_dir(&self) -> PathBuf {
        self.directory.join("payload")
    }

    /// Returns the path selected by the manifest.
    #[must_use]
    pub fn entrypoint(&self) -> PathBuf {
        self.payload_dir().join(Path::new(self.entrypoint.as_str()))
    }
}
