use serde::Deserialize;

use super::super::VersionReq;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireView {
    pub(super) point: String,
    pub(super) id: String,
    pub(super) title: String,
    #[serde(default)]
    pub(super) order: i32,
    pub(super) contract: Option<VersionReq>,
    pub(super) ir: Vec<u8>,
}
