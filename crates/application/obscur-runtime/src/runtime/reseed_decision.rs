#[doc = include_str!("../../docs/api/runtime/controlled_reseed.doc/ReseedDecision.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReseedDecision {
    /// Leave the session exactly as it is — buffer, caret, selection and
    /// watermark. A static or uncontrolled field, or a frame that rewrote
    /// something else.
    Keep,
    /// Advance the session's [`mirror`](super::TextEditSession::mirror) watermark to this
    /// retained value and keep the live buffer + caret: the session's **own** edit
    /// has come back through the app (a passthrough controlled input).
    Mirror(String),
    /// Reseed the buffer with this value, caret preserved (clamped into it): the
    /// value the field displays is not the value the app is holding, so the app's
    /// wins. The runtime re-derives the caret-follow scroll afterwards.
    Reseed(String),
}
