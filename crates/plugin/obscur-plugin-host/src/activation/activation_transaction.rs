//! `ActivationTransaction` (skeleton stub for the spec-defined entity; not yet implemented).

use std::collections::BTreeSet;
use std::panic::{AssertUnwindSafe, catch_unwind};

use obscur_plugin_api::{plugin::PluginRegistration, status::PluginApiError};
use obscur_plugin_model::identity::PluginId;
use obscur_routing::router::RouteRegistry;

use crate::contribution::{ExtensionPointRegistry, MountedView};

/// Validates every staged contribution before any of them becomes visible.
pub struct ActivationTransaction;

impl ActivationTransaction {
    pub(crate) fn validate(
        owner: &PluginId,
        staged: PluginRegistration,
        points: &ExtensionPointRegistry,
        routes: &RouteRegistry,
    ) -> Result<Vec<MountedView>, PluginApiError> {
        let views = staged.into_views();
        let mut routes = routes.clone();
        let mut seen = BTreeSet::new();
        let mut mounted = Vec::with_capacity(views.len());
        for view in views {
            if view
                .definition()
                .contract()
                .into_iter()
                .chain(view.manifest_contract())
                .any(|required| !points.supports(view.point(), required))
            {
                continue;
            }
            if !seen.insert((view.point().clone(), view.definition().id().clone())) {
                let id = format!(
                    "{}/{}/{}",
                    view.point().id(),
                    owner.as_str(),
                    view.definition().id().as_str()
                );
                return Err(PluginApiError::DuplicateContribution(id.into_boxed_str()));
            }
            let route = catch_unwind(AssertUnwindSafe(|| {
                points.route(view.point(), owner, view.definition())
            }))
            .map_err(|_| PluginApiError::Panicked)??;
            let view = MountedView::new(owner, view, route);
            if let (Some(route), Some(pattern)) = (view.route(), view.pattern()) {
                if !pattern.is_static() {
                    return Err(PluginApiError::Rejected(
                        format!("plugin view route must be static: {}", route.path()).into(),
                    ));
                }
                if routes.resolve(route).is_some() {
                    return Err(PluginApiError::RouteConflict(route.path().into()));
                }
                routes.register(pattern.clone(), view.global_id());
            }
            mounted.push(view);
        }
        Ok(mounted)
    }
}
