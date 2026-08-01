#![doc = include_str!("caret_visibility.doc/_module.md")]

use crate::runtime::CaretScroll;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("caret_visibility.doc/update_caret_scroll.md")]
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

    #[doc = include_str!("caret_visibility.doc/update_multiline_caret_scroll.md")]
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
