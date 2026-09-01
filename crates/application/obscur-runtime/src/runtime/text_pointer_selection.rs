#![doc = include_str!("text_pointer_selection.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_text::editing::{EditCommand, EditDirection, NavigationUnit};
use obscur_text::selection::TextSelection;
use obscur_tree::tree::RetainedTree;

use super::{EditableField, TextEditSession};
use crate::orchestration::{ComputedStyles, LayoutBoxes, SceneCoordinator, caret_offset_at_point};

#[doc = include_str!("text_pointer_selection.doc/TextPress.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextPress {
    #[doc = include_str!("text_pointer_selection.doc/Ignored.md")]
    Ignored,
    #[doc = include_str!("text_pointer_selection.doc/Caret.md")]
    Caret {
        #[doc = include_str!("text_pointer_selection.doc/field.md")]
        field: NodeId,
        #[doc = include_str!("text_pointer_selection.doc/offset.md")]
        offset: usize,
    },
    #[doc = include_str!("text_pointer_selection.doc/Range.md")]
    Range(TextSelection),
}

#[doc = include_str!("text_pointer_selection.doc/TextPointerSelection.md")]
pub(crate) struct TextPointerSelection<'a> {
    #[doc = include_str!("text_pointer_selection.doc/tree.md")]
    tree: &'a RetainedTree,
    #[doc = include_str!("text_pointer_selection.doc/computed.md")]
    computed: &'a ComputedStyles,
    #[doc = include_str!("text_pointer_selection.doc/boxes.md")]
    boxes: &'a LayoutBoxes,
    #[doc = include_str!("text_pointer_selection.doc/composer.md")]
    composer: &'a SceneCoordinator,
    #[doc = include_str!("text_pointer_selection.doc/session.md")]
    session: &'a mut TextEditSession,
}

impl<'a> TextPointerSelection<'a> {
    #[doc = include_str!("text_pointer_selection.doc/new.md")]
    pub(crate) fn new(
        tree: &'a RetainedTree,
        computed: &'a ComputedStyles,
        boxes: &'a LayoutBoxes,
        composer: &'a SceneCoordinator,
        session: &'a mut TextEditSession,
    ) -> Self {
        Self {
            tree,
            computed,
            boxes,
            composer,
            session,
        }
    }

    #[doc = include_str!("text_pointer_selection.doc/press.md")]
    pub(crate) fn press(&mut self, target: Option<NodeId>, point: Point, clicks: u32) -> TextPress {
        let Some(target) = target else {
            return TextPress::Ignored;
        };
        let Some(field) = EditableField::editable_ancestor(self.tree, target) else {
            return TextPress::Ignored;
        };
        // The session must already cover this field (sync runs on focus change; on
        // a repeat click into the already-focused field the session persists).
        if self.session.node() != field {
            return TextPress::Ignored;
        }
        let Some(offset) = self.offset_at(field, point) else {
            return TextPress::Ignored;
        };
        match clicks {
            // Double click: select the WORD under the pointer (word boundaries via
            // `NavigationUnit::Word`, spec §5.11). No drag is armed — the selection
            // is the whole word.
            2 => TextPress::Range(self.word_at(offset)),
            // Triple click (or beyond): select the whole LINE under the pointer
            // (logical line boundaries via `NavigationUnit::LineBoundary`; a
            // single-line field's whole span, spec §5.11).
            n if n >= 3 => TextPress::Range(self.line_at(offset)),
            // Single click: a collapsed caret at the click, and a potential
            // drag-selection anchored there.
            _ => TextPress::Caret { field, offset },
        }
    }

    #[doc = include_str!("text_pointer_selection.doc/drag.md")]
    pub(crate) fn drag(&self, field: NodeId, anchor: usize, point: Point) -> Option<TextSelection> {
        let current = self.offset_at(field, point)?;
        Some(TextSelection::new(anchor, current))
    }

    #[doc = include_str!("text_pointer_selection.doc/offset_at.md")]
    fn offset_at(&self, field: NodeId, point: Point) -> Option<usize> {
        if self.session.node() != field {
            return None;
        }
        let text_node = EditableField::first_text_descendant(self.tree, field)?;
        let text_box = self.boxes.get(&text_node)?;
        let origin = text_box.origin();
        // Undo the paint's `−scroll_left` translation: the run is drawn at
        // `origin.x + glyph_x − scroll_left`, so a pointer at screen `point.x`
        // sits at run-local `point.x − origin.x + scroll_left`. Adding the field's
        // scroll offset back lands a click on a scrolled field at the right byte
        // (the inverse of the caret paint, CSS Overflow 3 §3.1). `scroll_left`
        // is `0` on an unscrolled field ⇒ byte-identical to the pre-scroll inverse.
        // `local_y` picks the wrapped LINE (2D hit-test); a single-line field has
        // one line, so `local_y` is immaterial and the resolved byte is identical.
        let local_x = point.x - origin.x + self.session.scroll_left();
        let local_y = point.y - origin.y;
        Some(caret_offset_at_point(
            self.composer.text(),
            self.tree,
            self.computed,
            self.boxes,
            text_node,
            self.session.text(),
            text_box.size().width,
            local_x,
            local_y,
        ))
    }

    #[doc = include_str!("text_pointer_selection.doc/word_at.md")]
    fn word_at(&mut self, offset: usize) -> TextSelection {
        let buffer = self.session.buffer_mut();
        buffer.apply(EditCommand::SetSelection(TextSelection::caret(offset)));
        buffer.apply(EditCommand::Move {
            unit: NavigationUnit::Word,
            direction: EditDirection::Forward,
            extend: false,
        });
        buffer.apply(EditCommand::Move {
            unit: NavigationUnit::Word,
            direction: EditDirection::Backward,
            extend: true,
        });
        let range = buffer.selection().range();
        TextSelection::new(range.start, range.end)
    }

    #[doc = include_str!("text_pointer_selection.doc/line_at.md")]
    fn line_at(&mut self, offset: usize) -> TextSelection {
        let buffer = self.session.buffer_mut();
        buffer.apply(EditCommand::SetSelection(TextSelection::caret(offset)));
        buffer.apply(EditCommand::Move {
            unit: NavigationUnit::LineBoundary,
            direction: EditDirection::Backward,
            extend: false,
        });
        buffer.apply(EditCommand::Move {
            unit: NavigationUnit::LineBoundary,
            direction: EditDirection::Forward,
            extend: true,
        });
        let range = buffer.selection().range();
        TextSelection::new(range.start, range.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An empty frame: nothing is editable and no box is arranged, which is the
    /// state every one of these tests needs and none of them needs a runtime for.
    struct Frame {
        tree: RetainedTree,
        computed: ComputedStyles,
        boxes: LayoutBoxes,
        composer: SceneCoordinator,
    }

    impl Frame {
        fn new() -> Self {
            Self {
                tree: RetainedTree::new(),
                computed: ComputedStyles::default(),
                boxes: LayoutBoxes::default(),
                composer: SceneCoordinator::new(),
            }
        }

        fn selection<'a>(&'a self, session: &'a mut TextEditSession) -> TextPointerSelection<'a> {
            TextPointerSelection::new(
                &self.tree,
                &self.computed,
                &self.boxes,
                &self.composer,
                session,
            )
        }
    }

    #[test]
    fn a_press_on_nothing_selects_nothing() {
        let frame = Frame::new();
        let mut session = TextEditSession::seeded(NodeId::from_raw(1), "abc");
        assert_eq!(
            frame
                .selection(&mut session)
                .press(None, Point::new(0.0, 0.0), 1),
            TextPress::Ignored
        );
    }

    #[test]
    fn a_press_on_a_field_with_no_arranged_box_leaves_the_session_untouched() {
        let frame = Frame::new();
        let node = NodeId::from_raw(1);
        let mut session = TextEditSession::seeded(node, "abc");
        let before = session.selection();
        // Nothing in an empty tree is editable, so the press resolves no field —
        // and even a covered field with no laid-out text box resolves no offset.
        assert_eq!(
            frame
                .selection(&mut session)
                .press(Some(node), Point::new(4.0, 4.0), 1),
            TextPress::Ignored
        );
        assert_eq!(session.selection(), before);
    }

    #[test]
    fn a_drag_over_a_field_with_no_text_box_reports_no_selection() {
        let frame = Frame::new();
        let node = NodeId::from_raw(1);
        let mut session = TextEditSession::seeded(node, "abc");
        assert_eq!(
            frame
                .selection(&mut session)
                .drag(node, 0, Point::new(9.0, 1.0)),
            None
        );
    }

    #[test]
    fn a_word_bracket_normalises_to_a_forward_selection() {
        let frame = Frame::new();
        let node = NodeId::from_raw(1);
        let mut session = TextEditSession::seeded(node, "one two three");
        // The bracket walks the buffer's own word engine from a click inside the
        // middle word and reports it anchor-first, whichever way the walk ran.
        let word = frame.selection(&mut session).word_at(5);
        assert!(
            word.anchor() <= word.focus(),
            "the reported word selection is forward"
        );
        let range = word.range();
        assert_eq!(&"one two three"[range.start..range.end], "two");
    }

    #[test]
    fn a_line_bracket_of_a_single_line_field_is_its_whole_text() {
        let frame = Frame::new();
        let node = NodeId::from_raw(1);
        let mut session = TextEditSession::seeded(node, "one two");
        let line = frame.selection(&mut session).line_at(3);
        assert_eq!(line, TextSelection::new(0, 7));
    }
}
