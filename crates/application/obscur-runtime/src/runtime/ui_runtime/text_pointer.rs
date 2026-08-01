#![doc = include_str!("text_pointer.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_text::editing::EditCommand;
use obscur_text::selection::TextSelection;

use crate::runtime::{TextPointerSelection, TextPress};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("text_pointer.doc/begin_text_pointer.md")]
    pub(super) fn begin_text_pointer(&mut self, target: Option<NodeId>, point: Point) {
        // A new press always ends any prior drag; it is re-armed only below.
        self.pointers.set_text_drag(None);
        let clicks = self.pointers.click_count();
        let press = match self.editing.as_mut() {
            Some(session) => TextPointerSelection::new(
                &self.tree,
                &self.computed,
                &self.boxes,
                &self.scene_composer,
                session,
            )
            .press(target, point, clicks),
            None => TextPress::Ignored,
        };
        match press {
            TextPress::Ignored => {}
            // Single click: a collapsed caret at the click; `SetSelection` is
            // non-mutating, so `after_edit` (inside `apply_edit`) fires no
            // `on-input`, only a frame. Arms a potential drag-selection.
            TextPress::Caret { field, offset } => {
                self.apply_edit(EditCommand::SetSelection(TextSelection::caret(offset)));
                self.pointers.set_text_drag(Some((field, offset)));
            }
            // A word or a line: the whole range, and no drag armed.
            TextPress::Range(selection) => {
                self.apply_edit(EditCommand::SetSelection(selection));
            }
        }
    }

    #[doc = include_str!("text_pointer.doc/update_text_drag.md")]
    pub(super) fn update_text_drag(&mut self, point: Point) {
        let Some((field, anchor)) = self.pointers.text_drag() else {
            return;
        };
        let Some(session) = self.editing.as_mut() else {
            return;
        };
        let selection = TextPointerSelection::new(
            &self.tree,
            &self.computed,
            &self.boxes,
            &self.scene_composer,
            session,
        )
        .drag(field, anchor, point);
        if let Some(selection) = selection {
            self.apply_edit(EditCommand::SetSelection(selection));
        }
    }
}
