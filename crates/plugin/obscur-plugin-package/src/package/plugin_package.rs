//! PluginPackage (skeleton stub for the spec-defined entity; not yet implemented).

use obscur_plugin_model::manifest::PluginManifest;
use std::collections::{BTreeMap, BTreeSet};

use super::PackageEntry;
use crate::path::PackagePath;

/// An archive whose manifest, hashes, paths, and host policy have passed validation.
#[derive(Debug)]
pub struct PluginPackage {
    manifest: PluginManifest,
    manifest_file: PackageEntry,
    files: BTreeMap<PackagePath, PackageEntry>,
    entrypoint: PackagePath,
    host_features: BTreeSet<Box<str>>,
}

impl PluginPackage {
    pub(crate) const fn validated(
        manifest: PluginManifest,
        manifest_file: PackageEntry,
        files: BTreeMap<PackagePath, PackageEntry>,
        entrypoint: PackagePath,
        host_features: BTreeSet<Box<str>>,
    ) -> Self {
        Self {
            manifest,
            manifest_file,
            files,
            entrypoint,
            host_features,
        }
    }

    /// Returns the validated package metadata.
    #[must_use]
    pub const fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Lists the optional features requested by the manifest and offered by the validating host.
    pub fn available_host_features(&self) -> impl ExactSizeIterator<Item = &str> {
        self.host_features.iter().map(AsRef::as_ref)
    }

    /// Returns the verified file selected by the manifest.
    #[must_use]
    pub fn entrypoint(&self) -> &PackageEntry {
        &self.files[&self.entrypoint]
    }

    /// Returns the original manifest bytes from the validated package.
    #[must_use]
    pub fn manifest_bytes(&self) -> &[u8] {
        self.manifest_file.bytes()
    }

    pub(crate) fn files(&self) -> impl Iterator<Item = &PackageEntry> {
        self.files.values()
    }

    /// Looks up a hash-verified payload by its normalized path.
    #[must_use]
    pub fn file(&self, path: &PackagePath) -> Option<&PackageEntry> {
        self.files.get(path)
    }
}
