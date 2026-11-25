use serde::Deserialize;

use super::wire_view::WireView;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireUi {
    pub(super) version: u32,
    pub(super) revision: Option<u64>,
    pub(super) views: Vec<WireView>,
}
