//! The runtime-owned [`TextEditSession`]: the caret + selection + text state for
//! the currently focused editable field (spec §5.11, §6.7).

use obscur_identity::NodeId;
use obscur_text::editing::TextBuffer;
use obscur_text::ime::{Composition, Preedit};
use obscur_text::selection::TextSelection;
use obscur_text::span::TextRange;

use crate::orchestration::{EditingView, PreeditView};

#[doc = include_str!("../../docs/api/runtime/text_edit_session.doc/TextEditSession.md")]
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
    #[doc = include_str!("../../docs/api/runtime/text_edit_session.doc/scroll_left.md")]
    scroll_left: f32,
    #[doc = include_str!("../../docs/api/runtime/text_edit_session.doc/committed.md")]
    committed: String,
    #[doc = include_str!("../../docs/api/runtime/text_edit_session.doc/mirror.md")]
    mirror: String,
}

impl TextEditSession {
    /// Creates a session for `node` seeded with `initial_text` and the caret
    /// placed at the **end** of that text (the natural focus point for a field a
    /// user just tabbed/clicked into). The undo/redo history starts empty and no
    /// composition is active.
    pub(crate) fn seeded(node: NodeId, initial_text: impl Into<String>) -> Self {
        // The seed text is the `change` baseline (the value at focus) *and* the
        // buffer's initial content — clone it once for both.
        let initial = initial_text.into();
        let committed = initial.clone();
        // The seed value is also the initial retained-value watermark: at focus the
        // session mirrors exactly what the tree carries.
        let mirror = initial.clone();
        let mut buffer = TextBuffer::from_text(initial);
        // `from_text` seats the caret at 0; move it to the end of the seed text.
        let end = buffer.len();
        buffer.set_selection(TextSelection::caret(end));
        Self {
            node,
            buffer,
            composition: None,
            scroll_left: 0.0,
            committed,
            mirror,
        }
    }

    /// Replaces the session's text outright with `text`, caret at the end, clears
    /// the undo/redo history, and drops any active composition — an explicit
    /// re-seed from the app.
    pub(crate) fn reseed(&mut self, text: impl Into<String>) {
        // An explicit re-seed is a fresh value: it also resets the `change`
        // baseline, so a commit after re-seeding compares against the new value.
        let text = text.into();
        self.committed.clone_from(&text);
        // An explicit re-seed resets the retained-value watermark too: the app has
        // set a fresh value, so the tree is expected to carry it.
        self.mirror.clone_from(&text);
        self.buffer = TextBuffer::from_text(text);
        let end = self.buffer.len();
        self.buffer.set_selection(TextSelection::caret(end));
        self.composition = None;
        // A fresh value starts unscrolled; scroll-into-view re-derives the offset
        // on the next edit/caret move.
        self.scroll_left = 0.0;
    }

    #[doc = include_str!("../../docs/api/runtime/text_edit_session.doc/reseed_controlled.md")]
    pub(crate) fn reseed_controlled(&mut self, text: impl Into<String>) {
        let text = text.into();
        // Keep the caret where it was, clamped into the new value at a char
        // boundary (the new value may be shorter or re-cased).
        let caret = clamp_to_char_boundary(&text, self.buffer.selection().focus());
        self.mirror.clone_from(&text);
        self.buffer = TextBuffer::from_text(text);
        self.buffer.set_selection(TextSelection::caret(caret));
        self.composition = None;
        self.scroll_left = 0.0;
    }

    /// Returns the last retained value the session reconciled against — the
    /// watermark the controlled-input reseed compares the tree against.
    pub(crate) fn mirror(&self) -> &str {
        &self.mirror
    }

    /// Advances the retained-value watermark to `retained` without touching the
    /// buffer — used when the session's OWN edit reached the tree (a passthrough
    /// controlled input), so no reseed is needed but the watermark must track the
    /// value the tree now carries.
    pub(crate) fn set_mirror(&mut self, retained: String) {
        self.mirror = retained;
    }

    /// Returns the editable node this session is bound to.
    pub(crate) const fn node(&self) -> NodeId {
        self.node
    }

    /// Returns the field's current horizontal scroll offset (logical px, ≥ 0) —
    /// how far the run is scrolled left so the caret stays in the content box.
    pub(crate) const fn scroll_left(&self) -> f32 {
        self.scroll_left
    }

    /// Sets the field's horizontal scroll offset (logical px). The runtime clamps
    /// it into `0 ..= max(0, content_width − W)` before calling this, so the paint
    /// and click-inverse read a valid, in-range offset.
    pub(crate) const fn set_scroll_left(&mut self, scroll_left: f32) {
        self.scroll_left = scroll_left;
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

    /// Returns whether the field's live text differs from the last committed
    /// value — i.e. whether a commit (blur / single-line `Enter`) should fire a
    /// `change` event (WHATWG HTML §4.10.5.5: `change` fires only when the value
    /// actually changed since it was last committed / since focus).
    pub(crate) fn is_dirty(&self) -> bool {
        self.buffer.text() != self.committed
    }

    /// Marks the live text as the committed baseline, so a subsequent commit
    /// without further edits does not re-fire `change`. Called after a
    /// single-line `Enter` commit that keeps the field focused.
    pub(crate) fn mark_committed(&mut self) {
        self.committed = self.buffer.text().to_owned();
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
        self.composition
            .as_ref()
            .is_some_and(Composition::is_active)
    }

    /// Returns the active composition, if any — the source of the preedit the
    /// compose phase splices for display.
    pub(crate) fn composition(&self) -> Option<&Composition> {
        self.composition.as_ref()
    }

    #[doc = include_str!("../../docs/api/runtime/text_edit_session.doc/view.md")]
    pub(crate) fn view(&self) -> EditingView<'_> {
        // The field's live horizontal scroll offset, so compose translates the
        // run/caret/selection/preedit by `−scroll_left` and the field's
        // `overflow: hidden` clip hides the scrolled-out text (single-line
        // text-control scroll model, CSS Overflow 3 §3.1). `0.0` for a fitting
        // or wrapping field ⇒ byte-identical unscrolled paint. The measure pass
        // ignores `scroll_left` (it does not change the shaped string).
        let view = EditingView::new(self.node(), self.text(), self.selection())
            .with_scroll_left(self.scroll_left());
        // While composing, attach the live preedit so both passes splice it into
        // the field for DISPLAY — the buffer text is untouched (spec §5.12).
        match self.composition().filter(|c| c.is_active()) {
            Some(comp) => view.with_preedit(PreeditView::new(
                comp.preedit().text(),
                comp.anchor().start,
                comp.anchor().end,
                comp.preedit().caret(),
            )),
            None => view,
        }
    }
}

/// Clamps a byte `offset` into `text`: capped at `text.len()` and floored to the
/// nearest UTF-8 char boundary at or below it, so the result is always a valid
/// caret position in `text` (an offset from a previous, differently-sized value
/// may land past the end or mid-codepoint).
fn clamp_to_char_boundary(text: &str, offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }
    let mut i = offset;
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}
