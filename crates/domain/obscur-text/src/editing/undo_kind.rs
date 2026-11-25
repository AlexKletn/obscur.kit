#[doc = include_str!("../../docs/api/editing/undo_unit.doc/UndoKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UndoKind {
    /// Character insertion from ordinary typing — coalesces with adjacent typing.
    Typing,
    /// Backspace/Delete runs — coalesce with adjacent deletion.
    Deletion,
    /// A single IME composition — one composition is one undo unit (§5.12);
    /// never coalesces.
    Composition,
    /// A clipboard paste — atomic; never coalesces.
    Paste,
    /// Any other edit (programmatic, replace, etc.) — atomic; never coalesces.
    Other,
}

impl UndoKind {
    /// Returns whether two adjacent units of *this same* kind may merge into one
    /// undo step. Only [`Typing`](Self::Typing) and [`Deletion`](Self::Deletion)
    /// coalesce; the atomic kinds never do.
    #[must_use]
    pub const fn is_coalescing(&self) -> bool {
        matches!(self, Self::Typing | Self::Deletion)
    }
}
