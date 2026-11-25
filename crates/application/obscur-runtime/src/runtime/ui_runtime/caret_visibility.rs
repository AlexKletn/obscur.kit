#![doc = include_str!("../../../docs/api/runtime/ui_runtime/caret_visibility.doc/_module.md")]

use crate::runtime::CaretScroll;

use super::UiRuntime;

impl UiRuntime {
    /// Re-derives the focused field's horizontal caret-follow offset and seats it
    /// on the session — the write half of [`CaretScroll::horizontal`], which
    /// reports the offset but does not hold the session it belongs to.
    pub(super) fn update_caret_scroll(&mut self) {
        let Some(session) = self.editing.as_ref() else {
            return;
        };
        // `tree`, `computed`, `boxes`, `scene_composer` and `editing` are disjoint
        // fields: a legal split borrow.
        let scroll_left = CaretScroll::new(
            &self.tree,
            &self.computed,
            &self.boxes,
            &self.scene_composer,
        )
        .horizontal(session);
        if let (Some(scroll_left), Some(session)) = (scroll_left, self.editing.as_mut()) {
            session.set_scroll_left(scroll_left);
        }
    }

    /// Re-derives the focused multiline field's vertical caret-follow offset and
    /// seats it in the scroll map — the write half of [`CaretScroll::vertical`].
    pub(super) fn update_multiline_caret_scroll(&mut self) {
        let Some(session) = self.editing.as_ref() else {
            return;
        };
        let field = session.node();
        let stored = self.scroll.stored(field);
        let scroll_top = CaretScroll::new(
            &self.tree,
            &self.computed,
            &self.boxes,
            &self.scene_composer,
        )
        .vertical(session, stored);
        if let Some(scroll_top) = scroll_top {
            self.scroll.seat(field, scroll_top);
        }
    }
}
