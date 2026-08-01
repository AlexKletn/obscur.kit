#![doc = include_str!("composition.doc/_module.md")]

use crate::span::TextRange;

use super::Preedit;

#[doc = include_str!("composition.doc/CompositionState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositionState {
    #[doc = include_str!("composition.doc/Composing.md")]
    Composing,
    #[doc = include_str!("composition.doc/Committed.md")]
    Committed,
    #[doc = include_str!("composition.doc/Cancelled.md")]
    Cancelled,
}

#[doc = include_str!("composition.doc/Composition.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition {
    #[doc = include_str!("composition.doc/anchor.md")]
    anchor: TextRange,
    #[doc = include_str!("composition.doc/preedit.md")]
    preedit: Preedit,
    #[doc = include_str!("composition.doc/state.md")]
    state: CompositionState,
}

impl Composition {
    #[doc = include_str!("composition.doc/start.md")]
    #[must_use]
    pub fn start(anchor: TextRange) -> Self {
        Self {
            anchor,
            preedit: Preedit::empty(),
            state: CompositionState::Composing,
        }
    }

    #[doc = include_str!("composition.doc/update.md")]
    pub fn update(&mut self, preedit: Preedit) {
        if self.state == CompositionState::Composing {
            self.preedit = preedit;
        }
    }

    #[doc = include_str!("composition.doc/commit.md")]
    #[must_use]
    pub fn commit(&mut self) -> Box<str> {
        if self.state != CompositionState::Composing {
            return Box::from("");
        }
        let text = Box::from(self.preedit.text());
        self.preedit = Preedit::empty();
        self.state = CompositionState::Committed;
        text
    }

    #[doc = include_str!("composition.doc/cancel.md")]
    pub fn cancel(&mut self) {
        if self.state == CompositionState::Composing {
            self.preedit = Preedit::empty();
            self.state = CompositionState::Cancelled;
        }
    }

    #[doc = include_str!("composition.doc/is_active.md")]
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.state, CompositionState::Composing)
    }

    #[doc = include_str!("composition.doc/anchor_2.md")]
    #[must_use]
    pub const fn anchor(&self) -> TextRange {
        self.anchor
    }

    #[doc = include_str!("composition.doc/preedit_2.md")]
    #[must_use]
    pub const fn preedit(&self) -> &Preedit {
        &self.preedit
    }

    #[doc = include_str!("composition.doc/state_2.md")]
    #[must_use]
    pub const fn state(&self) -> CompositionState {
        self.state
    }

    #[doc = include_str!("composition.doc/validate_external_edit.md")]
    #[must_use]
    pub fn validate_external_edit(&self, edit: TextRange) -> bool {
        if self.state == CompositionState::Composing {
            !self.anchor.intersects(edit)
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_is_composing_with_empty_preedit() {
        let c = Composition::start(TextRange::new(3, 3));
        assert!(c.is_active());
        assert_eq!(c.state(), CompositionState::Composing);
        assert_eq!(c.anchor(), TextRange::new(3, 3));
        assert!(c.preedit().is_empty());
    }

    #[test]
    fn start_update_commit_flow() {
        let mut c = Composition::start(TextRange::new(0, 0));
        c.update(Preedit::new("nihon"));
        assert_eq!(c.preedit().text(), "nihon");
        assert!(c.is_active());

        let text = c.commit();
        assert_eq!(&*text, "nihon");
        assert_eq!(c.state(), CompositionState::Committed);
        assert!(!c.is_active());
        assert!(c.preedit().is_empty());
    }

    #[test]
    fn start_cancel_flow() {
        let mut c = Composition::start(TextRange::new(2, 5));
        c.update(Preedit::new("draft"));
        c.cancel();
        assert_eq!(c.state(), CompositionState::Cancelled);
        assert!(!c.is_active());
        assert!(c.preedit().is_empty());
    }

    #[test]
    fn update_ignored_after_terminal() {
        let mut c = Composition::start(TextRange::new(0, 0));
        let _ = c.commit();
        c.update(Preedit::new("late"));
        assert!(c.preedit().is_empty());
        assert_eq!(c.state(), CompositionState::Committed);
        // Committing again yields nothing and does not revert state.
        assert_eq!(&*c.commit(), "");
        assert_eq!(c.state(), CompositionState::Committed);
    }

    #[test]
    fn validate_external_edit_rejects_overlap_while_composing() {
        let c = Composition::start(TextRange::new(2, 6));
        // Overlapping edit is invalid.
        assert!(!c.validate_external_edit(TextRange::new(4, 8)));
        // Non-overlapping edits are valid.
        assert!(c.validate_external_edit(TextRange::new(6, 9)));
        assert!(c.validate_external_edit(TextRange::new(0, 2)));
    }

    #[test]
    fn validate_external_edit_allows_any_when_not_composing() {
        let mut c = Composition::start(TextRange::new(2, 6));
        let _ = c.commit();
        assert!(c.validate_external_edit(TextRange::new(4, 8)));
    }
}
