#![doc = include_str!("composition.doc/_module.md")]

use obscur_text::editing::EditCommand;
use obscur_text::ime::CompositionEvent;

use crate::frame::FrameReason;
use crate::runtime::{StructuralStates, TextEditSession};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("composition.doc/handle_ime.md")]
    pub fn handle_ime(&mut self, event: CompositionEvent) -> bool {
        // The follow-up the runtime owes after mutating the session — resolved
        // while the `session` borrow is live, then acted on below so the
        // `&mut self` calls (`request_frame`/`apply_edit`) do not overlap it.
        enum ImeFollowUp {
            #[doc = include_str!("composition.doc/Frame.md")]
            Frame,
            #[doc = include_str!("composition.doc/Commit.md")]
            Commit(obscur_text::span::TextRange, Box<str>),
            #[doc = include_str!("composition.doc/None.md")]
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

    #[doc = include_str!("composition.doc/composition_preedit.md")]
    #[must_use]
    pub fn composition_preedit(&self) -> Option<&str> {
        self.editing
            .as_ref()
            .and_then(TextEditSession::composition)
            .filter(|composition| composition.is_active())
            .map(|composition| composition.preedit().text())
    }
}
