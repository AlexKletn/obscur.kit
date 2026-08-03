#![doc = include_str!("key_edit_resolution.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_interaction::command::{KeyEvent, LogicalKey, Modifier, Modifiers, NamedKey};
use obscur_text::editing::{EditCommand, EditDirection, NavigationUnit};
use obscur_text::selection::TextSelection;
use obscur_tree::tree::RetainedTree;

use super::{EditableField, TextEditSession};
use crate::orchestration::{
    ComputedStyles, LayoutBoxes, SceneCoordinator, caret_line_move, caret_visual_line_bound,
};

#[doc = include_str!("key_edit_resolution.doc/KeyEdit.md")]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum KeyEdit {
    #[doc = include_str!("key_edit_resolution.doc/Declined.md")]
    Declined,
    #[doc = include_str!("key_edit_resolution.doc/Edit.md")]
    Edit(EditCommand),
    #[doc = include_str!("key_edit_resolution.doc/DeleteGrapheme.md")]
    DeleteGrapheme {
        #[doc = include_str!("key_edit_resolution.doc/direction.md")]
        direction: EditDirection,
        #[doc = include_str!("key_edit_resolution.doc/cluster.md")]
        cluster: Option<TextSelection>,
    },
    #[doc = include_str!("key_edit_resolution.doc/SelectAll.md")]
    SelectAll,
    #[doc = include_str!("key_edit_resolution.doc/History.md")]
    History {
        #[doc = include_str!("key_edit_resolution.doc/redo.md")]
        redo: bool,
    },
    #[doc = include_str!("key_edit_resolution.doc/Commit.md")]
    Commit,
}

#[doc = include_str!("key_edit_resolution.doc/KeyEditResolver.md")]
pub(crate) struct KeyEditResolver<'a> {
    #[doc = include_str!("key_edit_resolution.doc/tree.md")]
    tree: &'a RetainedTree,
    #[doc = include_str!("key_edit_resolution.doc/computed.md")]
    computed: &'a ComputedStyles,
    #[doc = include_str!("key_edit_resolution.doc/boxes.md")]
    boxes: &'a LayoutBoxes,
    #[doc = include_str!("key_edit_resolution.doc/composer.md")]
    composer: &'a SceneCoordinator,
    #[doc = include_str!("key_edit_resolution.doc/session.md")]
    session: &'a TextEditSession,
    #[doc = include_str!("key_edit_resolution.doc/modifiers.md")]
    modifiers: Modifiers,
}

impl<'a> KeyEditResolver<'a> {
    #[doc = include_str!("key_edit_resolution.doc/new.md")]
    pub(crate) fn new(
        tree: &'a RetainedTree,
        computed: &'a ComputedStyles,
        boxes: &'a LayoutBoxes,
        composer: &'a SceneCoordinator,
        session: &'a TextEditSession,
        modifiers: Modifiers,
    ) -> Self {
        Self {
            tree,
            computed,
            boxes,
            composer,
            session,
            modifiers,
        }
    }

    #[doc = include_str!("key_edit_resolution.doc/resolve.md")]
    pub(crate) fn resolve(&self, target: NodeId, event: &KeyEvent) -> KeyEdit {
        // An edit needs the session to be bound to this focus's editable ancestor.
        let node = self.session.node();
        if EditableField::editable_ancestor(self.tree, target) != Some(node) {
            return KeyEdit::Declined;
        }

        let shift = self.modifiers.contains(Modifier::Shift);
        let alt = self.modifiers.contains(Modifier::Alt);
        let cmd =
            self.modifiers.contains(Modifier::Ctrl) || self.modifiers.contains(Modifier::Meta);

        match event.logical() {
            // Editing chords: Ctrl/Meta (+Shift) letter — select-all / undo / redo.
            // Alt is excluded so a distinct Alt/AltGr chord is not swallowed.
            Some(LogicalKey::Character(c)) if cmd && !alt => match c.to_ascii_lowercase() {
                'a' => KeyEdit::SelectAll,
                'z' if !shift => KeyEdit::History { redo: false },
                'z' | 'y' => KeyEdit::History { redo: true },
                // Any other command chord is a potential app shortcut.
                _ => KeyEdit::Declined,
            },
            // A command modifier that is not an editing chord: not text.
            _ if cmd || alt => KeyEdit::Declined,
            // Plain printable character: insert (replaces the selection).
            Some(LogicalKey::Character(c)) => KeyEdit::Edit(EditCommand::insert(c.to_string())),
            Some(LogicalKey::Named(named)) => match named {
                NamedKey::Backspace | NamedKey::Delete => {
                    let backward = matches!(named, NamedKey::Backspace);
                    KeyEdit::DeleteGrapheme {
                        direction: if backward {
                            EditDirection::Backward
                        } else {
                            EditDirection::Forward
                        },
                        cluster: self
                            .grapheme_cluster_range(backward)
                            .map(|(start, end)| TextSelection::new(start, end)),
                    }
                }
                NamedKey::ArrowLeft => KeyEdit::Edit(EditCommand::Move {
                    unit: NavigationUnit::Grapheme,
                    direction: EditDirection::Backward,
                    extend: shift,
                }),
                NamedKey::ArrowRight => KeyEdit::Edit(EditCommand::Move {
                    unit: NavigationUnit::Grapheme,
                    direction: EditDirection::Forward,
                    extend: shift,
                }),
                NamedKey::Home => KeyEdit::Edit(self.line_boundary_move(node, false, shift)),
                NamedKey::End => KeyEdit::Edit(self.line_boundary_move(node, true, shift)),
                // Space produces a character in a focused text field. The
                // platform delivers it as a *named* key (`NamedKey::Space`, so a
                // control that is NOT being edited can use it for activation), so
                // the editing path — reached only with an active session — must
                // itself turn it into an inserted `' '` (replacing any selection);
                // otherwise the field would silently swallow every space.
                NamedKey::Space => KeyEdit::Edit(EditCommand::insert(" ")),
                // ArrowUp/ArrowDown move the caret between wrapped lines in a
                // MULTILINE field (spec §5.11). In a single-line `input` there is
                // no line to move to, so they are NOT consumed here (byte-identical
                // — they fall through to the shortcut/activation path as before).
                NamedKey::ArrowUp | NamedKey::ArrowDown => {
                    self.vertical_move(node, matches!(named, NamedKey::ArrowDown), shift)
                }
                // Enter inserts a newline in a MULTILINE field (CSS Text 3
                // §4.1.1 segment break); a single-line `input`'s Enter is NOT
                // consumed — it remains an activation/submit key (byte-identical).
                NamedKey::Enter if EditableField::is_multiline(self.tree, node) => {
                    KeyEdit::Edit(EditCommand::insert("\n"))
                }
                NamedKey::Enter => KeyEdit::Commit,
                // Escape / Tab / etc. remain activation or shortcut keys.
                _ => KeyEdit::Declined,
            },
            None => KeyEdit::Declined,
        }
    }

    #[doc = include_str!("key_edit_resolution.doc/vertical_move.md")]
    fn vertical_move(&self, node: NodeId, down: bool, shift: bool) -> KeyEdit {
        if !EditableField::is_multiline(self.tree, node) {
            return KeyEdit::Declined;
        }
        if self.session.node() != node {
            return KeyEdit::Declined;
        }
        let Some(text_node) = EditableField::first_text_descendant(self.tree, node) else {
            return KeyEdit::Declined;
        };
        let Some(text_box) = self.boxes.get(&text_node) else {
            return KeyEdit::Declined;
        };
        let content = self.session.text();
        let selection = self.session.selection();
        let focus = selection.focus();
        let new_focus = caret_line_move(
            self.composer.text(),
            self.tree,
            self.computed,
            self.boxes,
            text_node,
            content,
            text_box.size().width,
            focus,
            down,
        )
        // No line in that direction: clamp to the text end (Down on the last
        // line) or start (Up on the first line).
        .unwrap_or(if down { content.len() } else { 0 });
        let new_selection = if shift {
            TextSelection::new(selection.anchor(), new_focus)
        } else {
            TextSelection::caret(new_focus)
        };
        KeyEdit::Edit(EditCommand::SetSelection(new_selection))
    }

    #[doc = include_str!("key_edit_resolution.doc/grapheme_cluster_range.md")]
    fn grapheme_cluster_range(&self, backward: bool) -> Option<(usize, usize)> {
        use unicode_segmentation::GraphemeCursor;

        let session = self.session;
        let selection = session.selection();
        if !selection.is_collapsed() {
            return None;
        }
        let text = session.text();
        let focus = selection.focus();
        if backward {
            if focus == 0 {
                return None;
            }
            let boundary = GraphemeCursor::new(focus, text.len(), true)
                .prev_boundary(text, 0)
                .ok()??;
            spans_multiple_scalars(&text[boundary..focus]).then_some((boundary, focus))
        } else {
            if focus >= text.len() {
                return None;
            }
            let boundary = GraphemeCursor::new(focus, text.len(), true)
                .next_boundary(text, 0)
                .ok()??;
            spans_multiple_scalars(&text[focus..boundary]).then_some((focus, boundary))
        }
    }

    #[doc = include_str!("key_edit_resolution.doc/line_boundary_move.md")]
    fn line_boundary_move(&self, node: NodeId, to_end: bool, shift: bool) -> EditCommand {
        match self.visual_line_boundary(node, to_end) {
            Some(target) => {
                let selection = self.session.selection();
                let selection = if shift {
                    TextSelection::new(selection.anchor(), target)
                } else {
                    TextSelection::caret(target)
                };
                EditCommand::SetSelection(selection)
            }
            // No wrapped layout available: the logical line boundary (single-line /
            // hard-break field) — byte-identical to the historical path.
            None => EditCommand::Move {
                unit: NavigationUnit::LineBoundary,
                direction: if to_end {
                    EditDirection::Forward
                } else {
                    EditDirection::Backward
                },
                extend: shift,
            },
        }
    }

    #[doc = include_str!("key_edit_resolution.doc/visual_line_boundary.md")]
    fn visual_line_boundary(&self, node: NodeId, to_end: bool) -> Option<usize> {
        if self.session.node() != node {
            return None;
        }
        let text_node = EditableField::first_text_descendant(self.tree, node)?;
        let text_box = self.boxes.get(&text_node)?;
        let content = self.session.text();
        let focus = self.session.selection().focus();
        Some(caret_visual_line_bound(
            self.composer.text(),
            self.tree,
            self.computed,
            self.boxes,
            text_node,
            content,
            text_box.size().width,
            focus,
            to_end,
        ))
    }
}

#[doc = include_str!("key_edit_resolution.doc/spans_multiple_scalars.md")]
fn spans_multiple_scalars(cluster: &str) -> bool {
    cluster.chars().nth(1).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_interaction::command::PhysicalKey;

    /// An empty frame: no tree, no styles, no boxes, a default composer. Every
    /// test below drives the keymap against one of these and a session, which is
    /// the whole point of the extraction — none of them builds a runtime.
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
                computed: ComputedStyles::new(),
                boxes: LayoutBoxes::new(),
                composer: SceneCoordinator::new(),
            }
        }

        fn resolver<'a>(&'a self, session: &'a TextEditSession) -> KeyEditResolver<'a> {
            KeyEditResolver::new(
                &self.tree,
                &self.computed,
                &self.boxes,
                &self.composer,
                session,
                Modifiers::default(),
            )
        }
    }

    #[test]
    fn a_key_over_a_field_the_session_does_not_cover_is_declined() {
        let frame = Frame::new();
        let session = TextEditSession::seeded(NodeId::from_raw(1), "abc");
        // Nothing in an empty tree is editable, so no target resolves to the
        // session's node — every key falls through to the shortcut path.
        let event = KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character('x')));
        assert_eq!(
            frame
                .resolver(&session)
                .resolve(NodeId::from_raw(2), &event),
            KeyEdit::Declined
        );
    }

    #[test]
    fn a_multi_scalar_cluster_is_resolved_and_a_single_scalar_one_is_not() {
        let frame = Frame::new();
        // "e" + U+0301 COMBINING ACUTE: three bytes, two scalars, ONE cluster.
        let combining = TextEditSession::seeded(NodeId::from_raw(1), "e\u{301}");
        assert_eq!(
            frame.resolver(&combining).grapheme_cluster_range(true),
            Some((0, 3)),
            "backspace at the end deletes the whole two-scalar cluster"
        );

        let plain = TextEditSession::seeded(NodeId::from_raw(1), "ab");
        assert_eq!(
            frame.resolver(&plain).grapheme_cluster_range(true),
            None,
            "a single-scalar cluster defers to the buffer's own delete"
        );
        assert_eq!(
            frame.resolver(&plain).grapheme_cluster_range(false),
            None,
            "a caret at the end has nothing to delete forward"
        );
    }

    #[test]
    fn a_field_with_no_arranged_box_falls_back_to_the_logical_line_boundary() {
        let frame = Frame::new();
        let node = NodeId::from_raw(1);
        let session = TextEditSession::seeded(node, "abc");
        assert_eq!(
            frame.resolver(&session).visual_line_boundary(node, true),
            None
        );
        assert_eq!(
            frame
                .resolver(&session)
                .line_boundary_move(node, true, false),
            EditCommand::Move {
                unit: NavigationUnit::LineBoundary,
                direction: EditDirection::Forward,
                extend: false,
            },
            "with no wrapped layout, End is the buffer's logical line motion"
        );
    }

    #[test]
    fn a_single_line_field_never_consumes_a_vertical_arrow() {
        let frame = Frame::new();
        let node = NodeId::from_raw(1);
        let session = TextEditSession::seeded(node, "abc");
        // `is_multiline` is false for a node the tree does not carry as a
        // `textarea`, so ArrowDown falls through exactly as it does for an `input`.
        assert_eq!(
            frame.resolver(&session).vertical_move(node, true, false),
            KeyEdit::Declined
        );
    }
}
