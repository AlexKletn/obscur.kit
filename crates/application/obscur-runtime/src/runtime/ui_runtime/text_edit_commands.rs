#![doc = include_str!("text_edit_commands.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_interaction::command::KeyEvent;
use obscur_text::editing::{DeletionUnit, EditCommand};
use obscur_view::view::EVENT_INPUT;

use crate::frame::FrameReason;
use crate::runtime::{
    ComposeOnlyEdit, KeyEdit, KeyEditResolver, StructuralStates, TextEditSession,
};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("text_edit_commands.doc/try_text_edit.md")]
    pub(super) fn try_text_edit(&mut self, target: NodeId, event: &KeyEvent) -> bool {
        // An edit needs an active session bound to this focus's editable ancestor.
        let Some(session) = self.editing.as_ref() else {
            return false;
        };
        let node = session.node();
        let resolved = KeyEditResolver::new(
            &self.tree,
            &self.computed,
            &self.boxes,
            &self.scene_composer,
            session,
            self.modifiers,
        )
        .resolve(target, event);
        match resolved {
            KeyEdit::Declined => false,
            KeyEdit::Edit(command) => {
                self.apply_edit(command);
                true
            }
            KeyEdit::DeleteGrapheme { direction, cluster } => {
                // A multi-scalar cluster the resolver pre-resolved is deleted as
                // ONE selection edit; a readonly field never gets there, and a
                // single-scalar cluster falls through to the buffer's own delete
                // (which keeps its coalescing).
                if !StructuralStates::of(&self.tree).is_within_readonly(node)
                    && let Some(cluster) = cluster
                {
                    self.apply_edit(EditCommand::SetSelection(cluster));
                }
                self.apply_edit(EditCommand::Delete {
                    unit: DeletionUnit::Grapheme,
                    direction,
                });
                true
            }
            KeyEdit::SelectAll => {
                self.editing
                    .as_mut()
                    .expect("session present")
                    .buffer_mut()
                    .apply(EditCommand::SelectAll);
                self.after_edit(false);
                true
            }
            KeyEdit::History { redo } => {
                let changed = self.editing.as_mut().is_some_and(|s| {
                    let buffer = s.buffer_mut();
                    if redo { buffer.redo() } else { buffer.undo() }
                });
                self.after_edit(changed);
                true
            }
            KeyEdit::Commit => {
                self.commit_editable_change(true);
                false
            }
        }
    }

    #[doc = include_str!("text_edit_commands.doc/apply_edit.md")]
    pub(super) fn apply_edit(&mut self, command: EditCommand) {
        let mutating = command.is_mutating();
        // A **readonly** field (WHATWG HTML §attr-input-readonly) is focusable and
        // selectable but not editable: drop any text-mutating edit (insert / delete
        // / replace — this is the one choke point every typed insert, backspace/
        // delete, and IME commit flows through) while letting the non-mutating
        // caret/selection edits (Move / SetSelection / SelectAll) apply unchanged.
        if mutating
            && let Some(node) = self.editing.as_ref().map(TextEditSession::node)
            && StructuralStates::of(&self.tree).is_within_readonly(node)
        {
            return;
        }
        if let Some(session) = self.editing.as_mut() {
            session.buffer_mut().apply(command);
        }
        self.after_edit(mutating);
    }

    #[doc = include_str!("text_edit_commands.doc/after_edit.md")]
    fn after_edit(&mut self, mutating: bool) {
        let mut dispatched = false;
        if mutating
            && let Some((node, text)) = self
                .editing
                .as_ref()
                .map(|s| (s.node(), s.text().to_owned()))
            && let Some(command) = self
                .tree
                .get(node)
                .and_then(|n| n.command_for_event(EVENT_INPUT))
        {
            dispatched = self.dispatch_text_input(command, &text);
        }
        // Keep the caret in view: after any edit or caret move, re-derive the
        // single-line field's horizontal scroll offset so the caret stays inside
        // its content box (CSS Overflow 3 §3.1 scroll-into-view). A no-op for a
        // fitting or wrapping field.
        self.update_caret_scroll();
        // The vertical analogue for a MULTILINE field (`<textarea>`): re-derive the
        // overflow scroll container's `scroll_top` so the caret's LINE stays inside
        // the field's content box (CSS Overflow 3 §3.1 scroll-into-view). A no-op
        // for a single-line field or a textarea whose content fits.
        self.update_multiline_caret_scroll();
        // A dispatched `on-input` already requested a full frame (a reconcile is
        // needed for the mirror / reseed); only schedule our own when it did not.
        if !dispatched {
            let reason = if self.edit_is_compose_only(mutating) {
                FrameReason::TextEdit
            } else {
                FrameReason::Input
            };
            self.request_frame(reason);
        }
    }

    #[doc = include_str!("text_edit_commands.doc/edit_is_compose_only.md")]
    pub(super) fn edit_is_compose_only(&self, mutating: bool) -> bool {
        // A caret/selection change leaves the text — and therefore every box —
        // untouched: compose-only regardless of the field's shape.
        if !mutating {
            return true;
        }
        let Some(field) = self.editing.as_ref().map(TextEditSession::node) else {
            return false;
        };
        ComposeOnlyEdit::new(&self.tree, &self.computed).is_box_stable(field)
    }
}
