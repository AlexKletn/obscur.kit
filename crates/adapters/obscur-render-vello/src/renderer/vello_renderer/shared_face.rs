use std::sync::Arc;

#[doc = include_str!("../../../docs/api/renderer/vello_renderer.doc/SharedFace.md")]
pub(super) struct SharedFace(pub(super) Arc<[u8]>);

impl AsRef<[u8]> for SharedFace {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
