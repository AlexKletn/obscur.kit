#![doc = include_str!("../../../docs/api/runtime/ui_runtime/composition.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_text::editing::EditCommand;
use obscur_text::ime::{CompositionEvent, TextInputUpdate};
use obscur_text::span::TextRange;

use crate::frame::FrameReason;
use crate::runtime::{StructuralStates, TextEditSession};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/composition.doc/handle_ime.md")]
    pub fn handle_ime(&mut self, event: CompositionEvent) -> bool {
        // The follow-up the runtime owes after mutating the session — resolved
        // while the `session` borrow is live, then acted on below so the
        // `&mut self` calls (`request_frame`/`apply_edit`) do not overlap it.
        enum ImeFollowUp {
            /// Applied a preedit/state change: just request a paint frame.
            Frame,
            /// Committed: replace the anchor span with the text as one edit.
            Commit(obscur_text::span::TextRange, Box<str>),
            /// Out-of-order or no-op: nothing applied.
            None,
        }

        // No focused editable → nothing to compose against.
        let Some(node) = self.editing.as_ref().map(TextEditSession::node) else {
            return false;
        };
        // A **readonly** field cannot be edited (WHATWG HTML §attr-input-readonly):
        // IME composition — whose whole purpose is to compose text for insertion —
        // is inert on it. Drop the event before any preedit is anchored so nothing
        // is ever committed into the buffer. (Focus / caret / selection are
        // unaffected; only this mutation entry gates on readonly.)
        if StructuralStates::of(&self.tree).is_within_readonly(node) {
            return false;
        }
        // Re-borrow mutably to drive the composition (the session still exists — we
        // hold `&mut self` throughout — so this never fails; no panic path).
        let Some(session) = self.editing.as_mut() else {
            return false;
        };
        let follow_up = match event {
            CompositionEvent::Start => {
                session.start_composition();
                ImeFollowUp::Frame
            }
            CompositionEvent::Update(preedit) => {
                if session.is_composing() {
                    session.update_composition(preedit);
                    ImeFollowUp::Frame
                } else {
                    ImeFollowUp::None
                }
            }
            CompositionEvent::Commit(text) => match session.commit_composition() {
                Some(anchor) => ImeFollowUp::Commit(anchor, text),
                None => ImeFollowUp::None,
            },
            CompositionEvent::Cancel => {
                if session.is_composing() {
                    session.cancel_composition();
                    ImeFollowUp::Frame
                } else {
                    ImeFollowUp::None
                }
            }
        };
        match follow_up {
            ImeFollowUp::Frame => {
                self.update_caret_scroll();
                self.request_frame(FrameReason::Input);
                true
            }
            ImeFollowUp::Commit(anchor, text) => {
                // `apply_edit` runs `after_edit`: it fires the field's `on-input`
                // with the resulting text and requests an input frame. The preedit
                // was never inserted, so an empty anchor makes this an insert.
                self.apply_edit(EditCommand::Replace {
                    range: anchor,
                    text,
                });
                true
            }
            ImeFollowUp::None => false,
        }
    }

    /// The currently editable, writable node for native text input.
    #[must_use]
    pub fn text_input_target(&self) -> Option<NodeId> {
        let node = self.editing.as_ref()?.node();
        (!StructuralStates::of(&self.tree).is_within_readonly(node)).then_some(node)
    }

    /// Apply a native text-input batch to the same field that requested it.
    pub fn apply_text_input_update(&mut self, target: NodeId, update: TextInputUpdate) {
        if self.text_input_target() != Some(target) {
            return;
        }

        let Some(session) = self.editing.as_mut() else {
            return;
        };
        let directional = session.selection();
        let previous_composition = session.commit_composition();
        let selection = previous_composition.unwrap_or_else(|| directional.range());
        let text = session.text();
        let start = selection.start.saturating_sub(update.before as usize);
        let end = selection
            .end
            .saturating_add(update.after as usize)
            .min(text.len());
        if text.is_char_boundary(start) && text.is_char_boundary(end) {
            if let Some(commit) = update.commit {
                self.apply_edit(EditCommand::Replace {
                    range: TextRange::new(start, end),
                    text: commit,
                });
            } else if start != selection.start || end != selection.end {
                let preserved = text[selection.start..selection.end].into();
                self.apply_edit(EditCommand::Replace {
                    range: TextRange::new(start, end),
                    text: preserved,
                });
                if self.text_input_target() == Some(target) && !selection.is_empty() {
                    let (a, b) = (start, start + selection.len());
                    let (anchor, focus) = if directional.is_forward() {
                        (a, b)
                    } else {
                        (b, a)
                    };
                    self.apply_edit(EditCommand::SetSelection(
                        obscur_text::selection::TextSelection::new(anchor, focus),
                    ));
                }
            }
        }
        if self.text_input_target() == Some(target)
            && let Some(preedit) = update.preedit
        {
            self.handle_ime(CompositionEvent::Start);
            self.handle_ime(CompositionEvent::Update(preedit));
        } else if previous_composition.is_some() {
            self.update_caret_scroll();
            self.request_frame(FrameReason::Input);
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/composition.doc/composition_preedit.md")]
    #[must_use]
    pub fn composition_preedit(&self) -> Option<&str> {
        self.editing
            .as_ref()
            .and_then(TextEditSession::composition)
            .filter(|composition| composition.is_active())
            .map(|composition| composition.preedit().text())
    }
}
