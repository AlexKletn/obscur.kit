use std::collections::BTreeMap;

use obscur_identity::InternedSymbol;
use obscur_view::view::View;

use super::super::{ContributionId, ExtensionPoint, ViewContribution};
use super::plugin_ui_event::PluginUiEvent;

type ViewId = (ExtensionPoint, ContributionId);

pub(super) struct UiSnapshot {
    pub(super) revision: u64,
    pub(super) views: BTreeMap<ViewId, (ViewContribution, View)>,
    pub(super) events: BTreeMap<InternedSymbol, PluginUiEvent>,
}
