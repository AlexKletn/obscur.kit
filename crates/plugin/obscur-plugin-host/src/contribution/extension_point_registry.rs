use std::collections::{BTreeMap, btree_map::Entry};

use obscur_plugin_api::status::PluginApiError;
use obscur_plugin_model::{
    contribution::{ExtensionPoint, ViewContribution},
    identity::PluginId,
    manifest::{Version, VersionReq},
};
use obscur_routing::route::Route;

type MountPolicy =
    Box<dyn Fn(&PluginId, &ViewContribution) -> Result<Option<Route>, PluginApiError>>;

/// The application's published view integration points and mount policies.
#[derive(Default)]
pub struct ExtensionPointRegistry(BTreeMap<ExtensionPoint, (Option<Version>, MountPolicy)>);

impl ExtensionPointRegistry {
    /// Publishes a point with optional routing and host validation.
    /// Returns false if the point was already published.
    pub fn publish(
        &mut self,
        point: ExtensionPoint,
        mount: impl Fn(&PluginId, &ViewContribution) -> Result<Option<Route>, PluginApiError> + 'static,
    ) -> bool {
        self.publish_versioned(point, None, mount)
    }

    pub(crate) fn publish_versioned(
        &mut self,
        point: ExtensionPoint,
        version: Option<Version>,
        mount: impl Fn(&PluginId, &ViewContribution) -> Result<Option<Route>, PluginApiError> + 'static,
    ) -> bool {
        let Entry::Vacant(slot) = self.0.entry(point) else {
            return false;
        };
        slot.insert((version, Box::new(mount)));
        true
    }

    pub(crate) fn supports(&self, point: &ExtensionPoint, required: &VersionReq) -> bool {
        self.0
            .get(point)
            .and_then(|(version, _)| version.as_ref())
            .is_some_and(|version| required.matches(version))
    }

    pub(crate) fn route(
        &self,
        point: &ExtensionPoint,
        owner: &PluginId,
        view: &ViewContribution,
    ) -> Result<Option<Route>, PluginApiError> {
        let (_, mount) = self
            .0
            .get(point)
            .ok_or(PluginApiError::ExtensionUnavailable)?;
        mount(owner, view)
    }
}
