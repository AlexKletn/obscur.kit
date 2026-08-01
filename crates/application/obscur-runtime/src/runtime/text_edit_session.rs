#![doc = include_str!("text_edit_session.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_text::editing::TextBuffer;
use obscur_text::ime::{Composition, Preedit};
use obscur_text::selection::TextSelection;
use obscur_text::span::TextRange;

use crate::orchestration::{EditingView, PreeditView};

#[doc = include_str!("text_edit_session.doc/TextEditSession.md")]
pub(crate) struct TextEditSession {
    #[doc = include_str!("text_edit_session.doc/node.md")]
    node: NodeId,
    #[doc = include_str!("text_edit_session.doc/buffer.md")]
    buffer: TextBuffer,
    #[doc = include_str!("text_edit_session.doc/composition.md")]
    composition: Option<Composition>,
    #[doc = include_str!("text_edit_session.doc/scroll_left.md")]
    scroll_left: f32,
    #[doc = include_str!("text_edit_session.doc/committed.md")]
    committed: String,
    #[doc = include_str!("text_edit_session.doc/mirror.md")]
    mirror: String,
}

impl TextEditSession {
    #[doc = include_str!("text_edit_session.doc/seeded.md")]
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

    #[doc = include_str!("text_edit_session.doc/reseed.md")]
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

    #[doc = include_str!("text_edit_session.doc/reseed_controlled.md")]
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

    #[doc = include_str!("text_edit_session.doc/mirror_2.md")]
    pub(crate) fn mirror(&self) -> &str {
        &self.mirror
    }

    #[doc = include_str!("text_edit_session.doc/set_mirror.md")]
    pub(crate) fn set_mirror(&mut self, retained: String) {
        self.mirror = retained;
    }

    #[doc = include_str!("text_edit_session.doc/node_2.md")]
    pub(crate) const fn node(&self) -> NodeId {
        self.node
    }

    #[doc = include_str!("text_edit_session.doc/scroll_left_2.md")]
    pub(crate) const fn scroll_left(&self) -> f32 {
        self.scroll_left
    }

    #[doc = include_str!("text_edit_session.doc/set_scroll_left.md")]
    pub(crate) const fn set_scroll_left(&mut self, scroll_left: f32) {
        self.scroll_left = scroll_left;
    }

    #[doc = include_str!("text_edit_session.doc/buffer_mut.md")]
    pub(crate) fn buffer_mut(&mut self) -> &mut TextBuffer {
        &mut self.buffer
    }

    #[doc = include_str!("text_edit_session.doc/text.md")]
    pub(crate) fn text(&self) -> &str {
        self.buffer.text()
    }

    #[doc = include_str!("text_edit_session.doc/is_dirty.md")]
    pub(crate) fn is_dirty(&self) -> bool {
        self.buffer.text() != self.committed
    }

    #[doc = include_str!("text_edit_session.doc/mark_committed.md")]
    pub(crate) fn mark_committed(&mut self) {
        self.committed = self.buffer.text().to_owned();
    }

    #[doc = include_str!("text_edit_session.doc/selection.md")]
    pub(crate) fn selection(&self) -> TextSelection {
        self.buffer.selection()
    }

    #[doc = include_str!("text_edit_session.doc/start_composition.md")]
    pub(crate) fn start_composition(&mut self) {
        let anchor = self.buffer.selection().range();
        self.composition = Some(Composition::start(anchor));
    }

    #[doc = include_str!("text_edit_session.doc/update_composition.md")]
    pub(crate) fn update_composition(&mut self, preedit: Preedit) {
        if let Some(composition) = self.composition.as_mut() {
            composition.update(preedit);
        }
    }

    #[doc = include_str!("text_edit_session.doc/commit_composition.md")]
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

    #[doc = include_str!("text_edit_session.doc/cancel_composition.md")]
    pub(crate) fn cancel_composition(&mut self) {
        if let Some(composition) = self.composition.as_mut() {
            composition.cancel();
        }
        self.composition = None;
    }

    #[doc = include_str!("text_edit_session.doc/is_composing.md")]
    pub(crate) fn is_composing(&self) -> bool {
        self.composition
            .as_ref()
            .is_some_and(Composition::is_active)
    }

    #[doc = include_str!("text_edit_session.doc/composition_2.md")]
    pub(crate) fn composition(&self) -> Option<&Composition> {
        self.composition.as_ref()
    }

    #[doc = include_str!("text_edit_session.doc/view.md")]
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

#[doc = include_str!("text_edit_session.doc/clamp_to_char_boundary.md")]
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
