use serde::Serialize;

/// Guest-visible event identified by its contribution and UI IR node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(super) struct PluginUiEvent {
    pub(super) point: String,
    pub(super) view: String,
    pub(super) node: u64,
    pub(super) event: String,
    pub(super) action: String,
}
