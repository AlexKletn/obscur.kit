//! Validates dependency declarations against the host inventory.

use std::collections::{BTreeMap, BTreeSet};

use obscur_plugin_model::{
    identity::PluginId,
    manifest::{DependencyKind, PluginManifest},
};

use super::{DependencyConflict, DependencyGraph};

/// Validates dependency declarations against the host inventory.
#[derive(Debug)]
pub struct DependencyResolver;

impl DependencyResolver {
    /// Validates the supplied inventory and plans its provider-first activation order.
    /// The inventory has exactly one version per plugin ID.
    pub fn resolve<'a>(
        manifests: impl IntoIterator<Item = &'a PluginManifest>,
    ) -> Result<DependencyGraph<'a>, DependencyConflict> {
        let mut inventory = BTreeMap::new();
        for manifest in manifests {
            if inventory.insert(&manifest.id, manifest).is_some() {
                return Err(DependencyConflict::DuplicatePlugin(manifest.id.clone()));
            }
        }

        let mut providers = BTreeMap::new();
        for (&consumer, &manifest) in &inventory {
            let mut declared = BTreeSet::new();
            let mut present = Vec::new();
            for dependency in &manifest.dependencies {
                if !declared.insert(&dependency.id) {
                    return Err(DependencyConflict::DuplicateDependency {
                        consumer: consumer.clone(),
                        provider: dependency.id.clone(),
                    });
                }

                match inventory.get_key_value(&dependency.id) {
                    Some((&id, provider)) if dependency.version.matches(&provider.version) => {
                        present.push(id);
                    }
                    Some((_, provider)) => {
                        return Err(DependencyConflict::IncompatibleVersion {
                            consumer: consumer.clone(),
                            dependency: dependency.clone(),
                            found: provider.version.clone(),
                        });
                    }
                    None if dependency.kind == DependencyKind::Optional => {}
                    None => {
                        return Err(DependencyConflict::MissingProvider {
                            consumer: consumer.clone(),
                            dependency: dependency.clone(),
                        });
                    }
                }
            }
            present.sort_unstable();
            providers.insert(consumer, present);
        }
        let order = activation_order(&providers)?;
        Ok(DependencyGraph {
            manifests: inventory,
            order,
        })
    }
}

fn activation_order<'a>(
    providers: &BTreeMap<&'a PluginId, Vec<&'a PluginId>>,
) -> Result<Vec<&'a PluginId>, DependencyConflict> {
    enum Visit<'a> {
        Enter(&'a PluginId),
        Leave(&'a PluginId),
    }

    let mut order = Vec::with_capacity(providers.len());
    let mut completed = BTreeSet::new();
    let mut visiting = BTreeMap::new();
    let mut path = Vec::new();
    let mut pending = Vec::new();

    // Use an explicit stack so a deep dependency chain does not recurse.
    for &root in providers.keys() {
        pending.push(Visit::Enter(root));
        while let Some(visit) = pending.pop() {
            match visit {
                Visit::Enter(id) if completed.contains(id) => {}
                Visit::Enter(id) => {
                    if let Some(start) = visiting.insert(id, path.len()) {
                        let cycle = path[start..].iter().copied().chain([id]).cloned().collect();
                        return Err(DependencyConflict::Cycle(cycle));
                    }
                    path.push(id);
                    pending.push(Visit::Leave(id));
                    pending.extend(providers[id].iter().rev().copied().map(Visit::Enter));
                }
                Visit::Leave(id) => {
                    path.pop();
                    visiting.remove(id);
                    completed.insert(id);
                    order.push(id);
                }
            }
        }
    }
    Ok(order)
}
