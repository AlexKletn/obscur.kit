#![doc = include_str!("text_editing.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_text::selection::TextSelection;
use obscur_view::view::EVENT_CHANGE;

use crate::orchestration::EditingView;
use crate::runtime::{
    ControlledReseed, EditableField, ReseedDecision, StructuralStates, TextEditSession,
};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("text_editing.doc/begin_text_editing.md")]
    pub fn begin_text_editing(&mut self, node: NodeId, initial_text: impl Into<String>) {
        // A disabled control accepts no editing session — a defensive guard mirroring
        // the pointer path, so even an explicit app call can't make a disabled field
        // editable.
        if StructuralStates::of(&self.tree).is_within_disabled(node) {
            return;
        }
        self.editing = Some(TextEditSession::seeded(node, initial_text));
    }

    #[doc = include_str!("text_editing.doc/set_editing_text.md")]
    pub fn set_editing_text(&mut self, text: impl Into<String>) {
        if let Some(session) = self.editing.as_mut() {
            session.reseed(text);
        }
    }

    #[doc = include_str!("text_editing.doc/sync_editing_to_focus.md")]
    pub(super) fn sync_editing_to_focus(&mut self) {
        let new_editable = self
            .focused
            .and_then(|f| EditableField::editable_ancestor(&self.tree, f));
        let current = self.editing.as_ref().map(TextEditSession::node);
        // Same field (or both absent): no focus transition across an editable —
        // leave the live session (caret/selection/baseline) exactly as it was.
        if current == new_editable {
            return;
        }
        // Focus is leaving `current` (an editable): blur → commit its `change`
        // with the committed value if it changed since focus.
        if current.is_some() {
            self.commit_editable_change(false);
        }
        match new_editable {
            Some(node) => {
                let text = EditableField::collect_text(&self.tree, node);
                self.editing = Some(TextEditSession::seeded(node, text));
            }
            None => self.editing = None,
        }
    }

    #[doc = include_str!("text_editing.doc/reseed_editing_from_retained.md")]
    pub(super) fn reseed_editing_from_retained(&mut self) {
        let Some(session) = self.editing.as_ref() else {
            return;
        };
        match ControlledReseed::decide(&self.tree, session) {
            ReseedDecision::Keep => {}
            ReseedDecision::Mirror(retained) => {
                if let Some(session) = self.editing.as_mut() {
                    session.set_mirror(retained);
                }
            }
            ReseedDecision::Reseed(value) => {
                if let Some(session) = self.editing.as_mut() {
                    session.reseed_controlled(value);
                }
                // Re-derive the caret scroll afterwards.
                //
                // NOTE — reported, not changed (see `docs/runtime-decomposition.md`,
                // "reported, not changed"): this re-derives only the HORIZONTAL
                // caret-follow offset, where [`after_edit`](Self::after_edit)
                // re-derives both axes. A controlled `<textarea>` whose value is
                // externally rewritten shorter therefore keeps a `scroll_top`
                // computed for the old, longer content. Fixing it is its own change,
                // with its own test.
                self.update_caret_scroll();
            }
        }
    }

    #[doc = include_str!("text_editing.doc/commit_editable_change.md")]
    pub(super) fn commit_editable_change(&mut self, rebaseline: bool) -> bool {
        let Some(session) = self.editing.as_ref() else {
            return false;
        };
        if !session.is_dirty() {
            return false;
        }
        let node = session.node();
        // A **readonly** field's value can never change (WHATWG HTML
        // §attr-input-readonly), so its `change` never fires. The session can never
        // become dirty on a readonly field (every mutation is dropped upstream), so
        // this is normally unreachable — but guard explicitly so a commit is a
        // hard no-op regardless of how the session got here.
        if StructuralStates::of(&self.tree).is_within_readonly(node) {
            return false;
        }
        let text = session.text().to_owned();
        if rebaseline && let Some(session) = self.editing.as_mut() {
            session.mark_committed();
        }
        match self
            .tree
            .get(node)
            .and_then(|n| n.command_for_event(EVENT_CHANGE))
        {
            Some(command) => self.dispatch_change(command, &text),
            None => false,
        }
    }

    #[doc = include_str!("text_editing.doc/focus_accepts_text.md")]
    #[must_use]
    pub fn focus_accepts_text(&self) -> bool {
        self.focused
            .and_then(|node| EditableField::editable_ancestor(&self.tree, node))
            .is_some()
    }

    #[doc = include_str!("text_editing.doc/editing_text.md")]
    #[must_use]
    pub fn editing_text(&self) -> Option<&str> {
        self.editing.as_ref().map(TextEditSession::text)
    }

    #[doc = include_str!("text_editing.doc/editing_selection.md")]
    #[must_use]
    pub fn editing_selection(&self) -> Option<TextSelection> {
        self.editing.as_ref().map(TextEditSession::selection)
    }

    #[doc = include_str!("text_editing.doc/editing_view.md")]
    pub(super) fn editing_view(&self) -> Option<EditingView<'_>> {
        self.editing.as_ref().map(TextEditSession::view)
    }
}
