#[doc = include_str!("../../docs/api/surface/layer_state.doc/FeedbackTag.md")]
pub(super) struct FeedbackTag {
    /// Which feedback this is, counted by the surface that asked.
    pub(super) seq: u64,
    #[doc = include_str!("../../docs/api/surface/layer_state.doc/commits_at_ask.md")]
    pub(super) commits_at_ask: u64,
}
