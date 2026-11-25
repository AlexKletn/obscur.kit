use std::panic::{AssertUnwindSafe, catch_unwind};

use obscur_plugin_api::{plugin::RegisteredView, status::PluginApiError};
use obscur_plugin_model::{
    contribution::{ExtensionPoint, ViewContribution},
    identity::PluginId,
};
use obscur_routing::route::{Route, RoutePattern};
use obscur_view::view::View;

/// A plugin view mounted at an application-defined integration point.
pub struct MountedView {
    owner: PluginId,
    registered: RegisteredView,
    route: Option<(Route, RoutePattern)>,
}

impl MountedView {
    pub(crate) fn new(owner: &PluginId, registered: RegisteredView, route: Option<Route>) -> Self {
        Self {
            owner: owner.clone(),
            registered,
            route: route.map(|route| {
                let pattern = RoutePattern::parse(route.path());
                (route, pattern)
            }),
        }
    }

    /// Returns the owning plugin.
    #[must_use]
    pub const fn owner(&self) -> &PluginId {
        &self.owner
    }

    /// Returns the application's integration point.
    #[must_use]
    pub const fn point(&self) -> &ExtensionPoint {
        self.registered.point()
    }

    /// Returns metadata supplied by the plugin.
    #[must_use]
    pub const fn definition(&self) -> &ViewContribution {
        self.registered.definition()
    }

    /// Returns the route assigned by the application, if any.
    #[must_use]
    pub fn route(&self) -> Option<&Route> {
        self.route.as_ref().map(|(route, _)| route)
    }

    /// Returns the unique route target for this point, plugin, and local ID.
    #[must_use]
    pub fn global_id(&self) -> String {
        format!(
            "{}/{}/{}",
            self.point().id(),
            self.owner.as_str(),
            self.definition().id().as_str()
        )
    }

    pub(crate) fn pattern(&self) -> Option<&RoutePattern> {
        self.route.as_ref().map(|(_, pattern)| pattern)
    }

    /// Builds the view, reporting a panic in the plugin's render callback.
    pub fn render(&self) -> Result<View, PluginApiError> {
        catch_unwind(AssertUnwindSafe(|| self.registered.render()))
            .map_err(|_| PluginApiError::Panicked)
    }
}
