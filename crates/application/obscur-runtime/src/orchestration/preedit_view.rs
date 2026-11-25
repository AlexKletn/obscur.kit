#[doc = include_str!("../../docs/api/orchestration/editing_view.doc/PreeditView.md")]
#[derive(Debug, Clone, Copy)]
pub struct PreeditView<'a> {
    /// The uncommitted composition text shown inline.
    pub text: &'a str,
    /// The byte offset in the field's buffer text where the preedit is spliced for
    /// display — the composition anchor's start.
    pub at: usize,
    /// The byte offset in the field's buffer text where the composition anchor
    /// ends — the end of the span the preedit replaces for display. Equal to
    /// [`at`](Self::at) when the anchor is collapsed (no selection under the
    /// composition).
    pub end: usize,
    /// The caret byte offset *within* [`text`](Self::text): where the IME wants
    /// the insertion point drawn inside the preedit.
    pub caret: usize,
}

impl<'a> PreeditView<'a> {
    /// Creates a preedit view for `text` replacing the composition anchor span
    /// `[at, end)` for display, with its caret at byte `caret` within `text`.
    #[must_use]
    pub(crate) const fn new(text: &'a str, at: usize, end: usize, caret: usize) -> Self {
        Self {
            text,
            at,
            end,
            caret,
        }
    }
}
