//! A validated plugin inventory and provider-first activation plan.

use std::collections::BTreeMap;

use obscur_plugin_model::{
    identity::PluginId,
    manifest::{DependencyManifest, PluginManifest},
};

use super::{DependencyConflict, DependencyResolver};

/// A validated plugin inventory and provider-first activation plan.
#[derive(Debug)]
pub struct DependencyGraph<'a> {
    pub(super) manifests: BTreeMap<&'a PluginId, &'a PluginManifest>,
    pub(super) order: Vec<&'a PluginId>,
}

impl<'a> DependencyGraph<'a> {
    /// Returns all plugins in a deterministic order with providers before consumers.
    #[must_use]
    pub fn activation_order(&self) -> &[&'a PluginId] {
        &self.order
    }

    /// Returns the resolved dependencies of a plugin, including their declared kinds.
    /// A missing optional provider has no edge in the graph.
    pub fn dependencies(
        &self,
        id: &PluginId,
    ) -> Option<impl Iterator<Item = &'a DependencyManifest> + '_> {
        let manifest = *self.manifests.get(id)?;
        Some(
            manifest
                .dependencies
                .iter()
                .filter(|dependency| self.manifests.contains_key(&dependency.id)),
        )
    }

    /// Checks a replacement or new manifest against the full inventory and returns
    /// the resulting order if all consumers and providers remain compatible.
    pub fn with_update<'b>(
        &self,
        replacement: &'b PluginManifest,
    ) -> Result<DependencyGraph<'b>, DependencyConflict>
    where
        'a: 'b,
    {
        let inventory = self
            .manifests
            .values()
            .copied()
            .filter(|current| current.id != replacement.id)
            .chain(std::iter::once(replacement));
        DependencyResolver::resolve(inventory)
    }
}
