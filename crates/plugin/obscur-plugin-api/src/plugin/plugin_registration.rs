use std::collections::BTreeMap;

use obscur_plugin_model::{
    contribution::{ExtensionPoint, ViewContribution},
    manifest::VersionReq,
};
use obscur_view::view::View;

/// Contributions staged during a plugin's initialization.
#[derive(Default)]
pub struct PluginRegistration {
    views: Vec<RegisteredView>,
}

use super::RegisteredView;

impl PluginRegistration {
    /// Stages a view for the chosen integration point.
    pub fn contribute_view(
        &mut self,
        point: ExtensionPoint,
        definition: ViewContribution,
        render: impl Fn() -> View + 'static,
    ) {
        self.views
            .push(RegisteredView::new(point, definition, render));
    }

    /// Applies host-validated package requirements to staged views. A view must satisfy the manifest range and any range it declared separately.
    pub fn apply_manifest_contracts(&mut self, contracts: &BTreeMap<Box<str>, VersionReq>) {
        for view in &mut self.views {
            view.set_manifest_contract(contracts.get(view.point().id()).cloned());
        }
    }

    /// Consumes the staged view registrations.
    #[must_use]
    pub fn into_views(self) -> Vec<RegisteredView> {
        self.views
    }
}
