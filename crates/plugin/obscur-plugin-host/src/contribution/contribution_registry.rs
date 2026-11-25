use obscur_plugin_model::{contribution::ExtensionPoint, identity::PluginId};

use super::MountedView;

/// Views published by active plugins.
#[derive(Default)]
pub struct ContributionRegistry {
    views: Vec<MountedView>,
}

impl ContributionRegistry {
    /// Returns all currently registered views.
    #[must_use]
    pub fn views(&self) -> &[MountedView] {
        &self.views
    }

    /// Iterates the views registered for a particular integration point.
    pub fn views_at<'a>(
        &'a self,
        point: &'a ExtensionPoint,
    ) -> impl Iterator<Item = &'a MountedView> {
        self.views.iter().filter(move |view| view.point() == point)
    }

    pub(crate) fn publish(&mut self, views: Vec<MountedView>) {
        self.views.extend(views);
        self.views.sort_by(|left, right| {
            left.point()
                .cmp(right.point())
                .then_with(|| left.definition().order().cmp(&right.definition().order()))
                .then_with(|| left.owner().cmp(right.owner()))
                .then_with(|| left.definition().id().cmp(right.definition().id()))
        });
    }

    pub(crate) fn take_views(&mut self, owner: &PluginId) -> Vec<MountedView> {
        self.views
            .extract_if(.., |view| view.owner() == owner)
            .collect()
    }

    pub(crate) fn withdraw(&mut self, owner: &PluginId) {
        self.views.retain(|view| view.owner() != owner);
    }
}
