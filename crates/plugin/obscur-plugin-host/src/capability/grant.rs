use std::time::Instant;

use obscur_plugin_model::identity::PluginPrincipal;

use super::ServiceScope;

pub(super) struct Grant {
    pub(super) principal: PluginPrincipal,
    pub(super) scope: ServiceScope,
    pub(super) expires: Option<Instant>,
    pub(super) parent: Option<u64>,
}
