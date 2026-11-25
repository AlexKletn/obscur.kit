//! Metadata-only inventory for plugin discovery and lazy activation.

use std::collections::{BTreeMap, btree_map::Entry};

use obscur_plugin_model::{
    identity::PluginId,
    manifest::{PluginManifest, Version, VersionReq},
};

/// Indexes manifest metadata by plugin ID and semantic version. Indexing and selection never execute a plugin.
#[derive(Debug, Default)]
pub struct PluginCatalog {
    manifests: BTreeMap<PluginId, BTreeMap<Version, PluginManifest>>,
}

impl PluginCatalog {
    /// Indexes manifest metadata. An existing ID and version retain their original entry.
    pub fn insert(&mut self, manifest: PluginManifest) -> bool {
        let versions = self.manifests.entry(manifest.id.clone()).or_default();
        let Entry::Vacant(slot) = versions.entry(manifest.version.clone()) else {
            return false;
        };
        slot.insert(manifest);
        true
    }

    /// Returns the highest catalogued version accepted by the requested range.
    #[must_use]
    pub fn select(&self, id: &PluginId, accepted: &VersionReq) -> Option<&PluginManifest> {
        self.manifests
            .get(id)?
            .iter()
            .rev()
            .find(|(version, _)| accepted.matches(version))
            .map(|(_, manifest)| manifest)
    }

    /// Lists all catalogued manifests in ID and version order.
    pub fn manifests(&self) -> impl Iterator<Item = &PluginManifest> {
        self.manifests.values().flat_map(BTreeMap::values)
    }

    /// Removes the specified version from the metadata inventory.
    pub fn remove(&mut self, id: &PluginId, version: &Version) -> Option<PluginManifest> {
        let versions = self.manifests.get_mut(id)?;
        let manifest = versions.remove(version)?;
        if versions.is_empty() {
            self.manifests.remove(id);
        }
        Some(manifest)
    }
}
