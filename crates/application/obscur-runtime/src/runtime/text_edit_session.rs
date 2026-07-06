//! The runtime-owned [`TextEditSession`]: the caret + selection + text state for
//! the currently focused editable field (spec §5.11, §6.7).

use obscur_identity::NodeId;
use obscur_text::editing::TextBuffer;
use obscur_text::selection::TextSelection;

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
}

impl TextEditSession {
    /// Creates a session for `node` seeded with `initial_text` and the caret
    /// placed at the **end** of that text (the natural focus point for a field a
    /// user just tabbed/clicked into). The undo/redo history starts empty.
    pub(crate) fn seeded(node: NodeId, initial_text: impl Into<String>) -> Self {
        let mut buffer = TextBuffer::from_text(initial_text);
        // `from_text` seats the caret at 0; move it to the end of the seed text.
        let end = buffer.len();
        buffer.set_selection(TextSelection::caret(end));
        Self { node, buffer }
    }

    /// Replaces the session's text outright with `text`, caret at the end, and
    /// clears the undo/redo history — an explicit re-seed from the app.
    pub(crate) fn reseed(&mut self, text: impl Into<String>) {
        self.buffer = TextBuffer::from_text(text);
        let end = self.buffer.len();
        self.buffer.set_selection(TextSelection::caret(end));
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
}
