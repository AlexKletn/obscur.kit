//! The per-frame [`EditingView`] the scene phase paints a caret + selection from.

use obscur_identity::NodeId;
use obscur_text::selection::TextSelection;

use super::PreeditView;

#[doc = include_str!("../../docs/api/orchestration/editing_view.doc/EditingView.md")]
#[derive(Debug, Clone, Copy)]
pub struct EditingView<'a> {
    /// The editable field node the session is bound to.
    pub node: NodeId,
    /// The session's current text — the source of truth painted for the field,
    /// overriding the (possibly lagging) retained text content.
    pub text: &'a str,
    /// The session's current caret/selection: collapsed is a caret, non-empty is
    /// a highlighted span.
    pub selection: TextSelection,
    /// The active IME preedit spliced into the field for DISPLAY when a
    /// composition is live (spec §5.12), or `None` when the field is not
    /// composing. When present, the compose phase draws `text` with the preedit
    /// spliced at [`PreeditView::at`], underlines the preedit span, and places the
    /// caret at the preedit's caret — none of which touches the buffer.
    pub preedit: Option<PreeditView<'a>>,
    #[doc = include_str!("../../docs/api/orchestration/editing_view.doc/scroll_left.md")]
    pub scroll_left: f32,
}

impl<'a> EditingView<'a> {
    /// Creates an editing view for `node` painting `text` with `selection` and no
    /// active preedit.
    #[must_use]
    pub(crate) const fn new(node: NodeId, text: &'a str, selection: TextSelection) -> Self {
        Self {
            node,
            text,
            selection,
            preedit: None,
            scroll_left: 0.0,
        }
    }

    /// Returns a copy with the active IME `preedit` attached (spec §5.12).
    #[must_use]
    pub(crate) const fn with_preedit(mut self, preedit: PreeditView<'a>) -> Self {
        self.preedit = Some(preedit);
        self
    }

    /// Returns a copy carrying the field's horizontal `scroll_left` offset
    /// (logical px, ≥ 0) — how far the run is scrolled left so the caret stays in
    /// the field's content box (CSS Overflow 3 §3.1). `0.0` leaves the paint
    /// byte-identical to the unscrolled path.
    #[must_use]
    pub(crate) const fn with_scroll_left(mut self, scroll_left: f32) -> Self {
        self.scroll_left = scroll_left;
        self
    }
}
