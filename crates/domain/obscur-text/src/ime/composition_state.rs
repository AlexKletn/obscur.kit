#[doc = include_str!("../../docs/api/ime/composition.doc/CompositionState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositionState {
    /// The composition is live; the preedit may still be updated.
    Composing,
    /// The composition was committed; its text was finalized.
    Committed,
    /// The composition was cancelled; nothing was committed.
    Cancelled,
}
