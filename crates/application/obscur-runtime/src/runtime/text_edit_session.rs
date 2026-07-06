//! The runtime-owned [`TextEditSession`]: the caret + selection + text state for
//! the currently focused editable field (spec §5.11, §6.7).

use obscur_identity::NodeId;
use obscur_text::editing::TextBuffer;
use obscur_text::ime::{Composition, Preedit};
use obscur_text::selection::TextSelection;
use obscur_text::span::TextRange;

/// The runtime's editing session for the focused text field.
///
/// Unlike the append-only per-keystroke model it replaces, the runtime is now
/// the **source of truth** for the focused field's text, caret and selection
/// (an *uncontrolled* input): a [`TextEditSession`] owns a deterministic
/// [`TextBuffer`] (spec §5.11) that the keymap in
/// [`UiRuntime::handle_key`](super::UiRuntime::handle_key) drives with
/// [`EditCommand`](obscur_text::editing::EditCommand)s. The app is notified of
/// the *resulting text* after each mutating edit; step-2 rendering reads the
/// live [`text`](Self::text)/[`selection`](Self::selection) back to paint the
/// caret and selection highlight.
///
/// A session is bound to one [`NodeId`] — the editable node whose ancestor chain
/// binds [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT). When focus leaves that
/// node the session is dropped; when it lands on a *different* editable a fresh
/// session is seeded (see
/// [`UiRuntime::begin_text_editing`](super::UiRuntime::begin_text_editing)).
pub(crate) struct TextEditSession {
    /// The editable node this session edits (the ancestor binding `EVENT_INPUT`).
    node: NodeId,
    /// The pure editing engine holding the field's text, selection and undo/redo.
    buffer: TextBuffer,
    /// The active IME composition anchored in `buffer`, if the field is composing
    /// (spec §5.12). While `Some(_)` and active a preedit is shown inline (spliced
    /// only for display) but kept OUT of `buffer` until it commits; a commit
    /// replaces the anchor span in `buffer` with the committed text as one edit
    /// (one composition = one undo unit).
    composition: Option<Composition>,
}

impl TextEditSession {
    /// Creates a session for `node` seeded with `initial_text` and the caret
    /// placed at the **end** of that text (the natural focus point for a field a
    /// user just tabbed/clicked into). The undo/redo history starts empty and no
    /// composition is active.
    pub(crate) fn seeded(node: NodeId, initial_text: impl Into<String>) -> Self {
        let mut buffer = TextBuffer::from_text(initial_text);
        // `from_text` seats the caret at 0; move it to the end of the seed text.
        let end = buffer.len();
        buffer.set_selection(TextSelection::caret(end));
        Self {
            node,
            buffer,
            composition: None,
        }
    }

    /// Replaces the session's text outright with `text`, caret at the end, clears
    /// the undo/redo history, and drops any active composition — an explicit
    /// re-seed from the app.
    pub(crate) fn reseed(&mut self, text: impl Into<String>) {
        self.buffer = TextBuffer::from_text(text);
        let end = self.buffer.len();
        self.buffer.set_selection(TextSelection::caret(end));
        self.composition = None;
    }

    /// Returns the editable node this session is bound to.
    pub(crate) const fn node(&self) -> NodeId {
        self.node
    }

    /// Returns a mutable reference to the underlying editing engine, so the
    /// runtime can [`apply`](TextBuffer::apply) an
    /// [`EditCommand`](obscur_text::editing::EditCommand) or drive undo/redo.
    pub(crate) fn buffer_mut(&mut self) -> &mut TextBuffer {
        &mut self.buffer
    }

    /// Returns the session's current text.
    pub(crate) fn text(&self) -> &str {
        self.buffer.text()
    }

    /// Returns the session's current selection (a collapsed selection is a plain
    /// caret).
    pub(crate) fn selection(&self) -> TextSelection {
        self.buffer.selection()
    }

    /// Starts an IME composition anchored at the current selection/caret (spec
    /// §5.12 "start"), replacing any prior composition. The preedit begins empty
    /// and is shown inline (not inserted) at the anchor; a subsequent commit
    /// replaces the anchor span with the committed text.
    pub(crate) fn start_composition(&mut self) {
        let anchor = self.buffer.selection().range();
        self.composition = Some(Composition::start(anchor));
    }

    /// Replaces the live composition's preedit with `preedit` (spec §5.12
    /// "update"). A no-op when no composition is active — the preedit is stored on
    /// the composition, never in the buffer.
    pub(crate) fn update_composition(&mut self, preedit: Preedit) {
        if let Some(composition) = self.composition.as_mut() {
            composition.update(preedit);
        }
    }

    /// Commits the live composition (spec §5.12 "commit"): marks it committed,
    /// clears it, and returns the buffer byte range the caller must replace with
    /// the committed text (the anchor span). Returns `None` when no composition is
    /// active; the buffer replacement itself (one undo unit) is the caller's job.
    pub(crate) fn commit_composition(&mut self) -> Option<TextRange> {
        let composition = self.composition.as_mut()?;
        if !composition.is_active() {
            self.composition = None;
            return None;
        }
        let anchor = composition.anchor();
        // Advance the composition's own lifecycle to `Committed` (clears preedit);
        // the returned text is discarded — the runtime commits the IME-provided
        // committed text over `anchor` as a single buffer edit.
        let _ = composition.commit();
        self.composition = None;
        Some(anchor)
    }

    /// Cancels the live composition (spec §5.12 "cancel"): drops the preedit with
    /// no buffer change. A no-op when no composition is active.
    pub(crate) fn cancel_composition(&mut self) {
        if let Some(composition) = self.composition.as_mut() {
            composition.cancel();
        }
        self.composition = None;
    }

    /// Returns whether an IME composition is currently live (composing).
    pub(crate) fn is_composing(&self) -> bool {
        self.composition.as_ref().is_some_and(Composition::is_active)
    }

    /// Returns the active composition, if any — the source of the preedit the
    /// compose phase splices for display.
    pub(crate) fn composition(&self) -> Option<&Composition> {
        self.composition.as_ref()
    }
}
