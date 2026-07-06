//! The per-frame [`EditingView`] the scene phase paints a caret + selection from.

use obscur_identity::NodeId;
use obscur_text::selection::TextSelection;

/// A borrowed snapshot of the runtime's active text-editing session, threaded
/// into [`SceneCoordinator::compose`](super::SceneCoordinator::compose) so the
/// focused editable paints from the **live** session state (spec §5.11, §6.7).
///
/// Step 1 gave the runtime an editing session (owning the focused field's text,
/// caret and selection); this is the read-only view the compose phase consumes to
/// (a) render the editing field's text from the session `text` — so a typed
/// character is visible immediately, even before the retained
/// [`NodeKind::Text`](obscur_tree::node::NodeKind::Text) content catches up — and
/// (b) paint the caret line and selection highlight from `selection`.
///
/// `node` is the **editable field** the session is bound to (the ancestor that
/// binds [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT)), not necessarily the
/// `Text` leaf that carries the glyphs: the compose walk treats the field's `Text`
/// descendant as the editing text. `text` borrows the session buffer's current
/// string and `selection` is its caret/selection (a collapsed selection is a
/// plain caret). The runtime builds this from `self.editing` before each compose
/// and passes `None` when no session is active (the byte-identical, no-caret
/// path).
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
}

/// A borrowed snapshot of the active IME preedit spliced into the editing field
/// for DISPLAY (spec §5.12 IME).
///
/// The preedit is *not* committed text: it is the uncommitted composition string
/// shown inline at [`at`](Self::at) (a byte offset into the field's buffer text,
/// the composition anchor's start) and rendered distinctly (an underline), but it
/// is **never** inserted into the buffer until the composition commits. The
/// compose phase splices `text` into the buffer text only to paint it.
#[derive(Debug, Clone, Copy)]
pub struct PreeditView<'a> {
    /// The uncommitted composition text shown inline.
    pub text: &'a str,
    /// The byte offset in the field's buffer text where the preedit is spliced for
    /// display — the composition anchor's start.
    pub at: usize,
    /// The caret byte offset *within* [`text`](Self::text): where the IME wants
    /// the insertion point drawn inside the preedit.
    pub caret: usize,
}

impl<'a> PreeditView<'a> {
    /// Creates a preedit view for `text` spliced at byte offset `at` with its
    /// caret at byte `caret` within `text`.
    #[must_use]
    pub(crate) const fn new(text: &'a str, at: usize, caret: usize) -> Self {
        Self { text, at, caret }
    }
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
        }
    }

    /// Returns a copy with the active IME `preedit` attached (spec §5.12).
    #[must_use]
    pub(crate) const fn with_preedit(mut self, preedit: PreeditView<'a>) -> Self {
        self.preedit = Some(preedit);
        self
    }
}
