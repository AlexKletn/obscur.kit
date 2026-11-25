//! A verified manifest snapshot and its source archive.

use std::path::{Path, PathBuf};

use obscur_plugin_model::manifest::PluginManifest;
use obscur_plugin_package::{
    package::PluginPackage,
    validation::{PackageError, PackageValidator},
};

/// A discovered package that is revalidated before it is loaded for activation.
#[derive(Debug)]
pub struct DiscoveredPlugin {
    pub(super) path: PathBuf,
    pub(super) manifest: PluginManifest,
}

impl DiscoveredPlugin {
    /// Returns the archive path captured during discovery.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the verified manifest snapshot used for discovery.
    #[must_use]
    pub const fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Reloads and validates the archive before activation. A changed manifest
    /// requires rediscovery before the replacement can be used.
    pub fn load(&self, validator: &PackageValidator) -> Result<PluginPackage, PackageError> {
        let package = validator.load_path(&self.path)?;
        if package.manifest() != &self.manifest {
            return Err(PackageError::Manifest(
                "plugin changed after discovery".into(),
            ));
        }
        Ok(package)
    }
}
