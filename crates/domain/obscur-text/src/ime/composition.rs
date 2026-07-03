//! The active IME composition session anchored in the target buffer
//! (spec §5.12 IME).

use crate::span::TextRange;

use super::Preedit;

/// The lifecycle state of a [`Composition`] (spec §5.12).
///
/// A composition begins [`Composing`](Self::Composing) and ends exactly once, in
/// either [`Committed`](Self::Committed) (the preedit was finalized into the
/// buffer) or [`Cancelled`](Self::Cancelled) (the composition was abandoned and
/// the anchored range left untouched).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositionState {
    /// The composition is live; the preedit may still be updated.
    Composing,
    /// The composition was committed; its text was finalized.
    Committed,
    /// The composition was cancelled; nothing was committed.
    Cancelled,
}

/// An active IME composition session anchored in the target buffer (spec §5.12).
///
/// A composition ties an [`anchor`](Self::anchor) — the byte range in the buffer
/// the composition occupies and will replace — to a live [`preedit`](Self::preedit)
/// and a [`state`](Self::state). The anchor is established at the primary caret
/// when the session [`start`](Self::start)s (multi-selection is restricted while
/// composing, so there is a single anchor); while
/// [`Composing`](CompositionState::Composing) the preedit is shown inline at that
/// range.
///
/// The typed lifecycle mirrors §5.12: [`start`](Self::start) →
/// [`update`](Self::update)\* → [`commit`](Self::commit) or
/// [`cancel`](Self::cancel). Per §5.12 "one composition is one undo unit"; that
/// undo grouping is enforced by the editing engine, which wraps the commit in a
/// single [`UndoKind::Composition`](crate::editing::UndoKind::Composition) unit.
/// This type only models the composition *state* — it performs no buffer edits;
/// the caller applies [`commit`](Self::commit)'s returned text as a replacement
/// over the anchor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition {
    /// The byte range in the target buffer the composition occupies / replaces.
    anchor: TextRange,
    /// The uncommitted preedit shown at the anchor.
    preedit: Preedit,
    /// The lifecycle state.
    state: CompositionState,
}

impl Composition {
    /// Starts a composition anchored at `anchor` (spec §5.12 "start").
    ///
    /// The session begins [`Composing`](CompositionState::Composing) with an
    /// empty preedit. Anchoring at the primary caret's range here also
    /// establishes the one-undo-unit scope the editing engine will use.
    #[must_use]
    pub fn start(anchor: TextRange) -> Self {
        Self {
            anchor,
            preedit: Preedit::empty(),
            state: CompositionState::Composing,
        }
    }

    /// Replaces the preedit with `preedit`, keeping the state
    /// [`Composing`](CompositionState::Composing) (spec §5.12 "update").
    ///
    /// Ignored (a no-op) when the composition is no longer composing, so a stray
    /// update after commit/cancel cannot resurrect a finished session.
    pub fn update(&mut self, preedit: Preedit) {
        if self.state == CompositionState::Composing {
            self.preedit = preedit;
        }
    }

    /// Commits the composition and returns the text to insert: the preedit text
    /// at commit time (spec §5.12 "commit").
    ///
    /// Marks the state [`Committed`](CompositionState::Committed) and clears the
    /// preedit. The buffer-side replacement over the [`anchor`](Self::anchor) and
    /// its undo-unit handling live in the editing engine, not here. Committing an
    /// already-finished composition returns an empty string and leaves the state
    /// unchanged.
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

    /// Cancels the composition (spec §5.12 "cancel").
    ///
    /// Marks the state [`Cancelled`](CompositionState::Cancelled) and clears the
    /// preedit; the anchored range is left untouched by the caller. Ignored when
    /// the composition is no longer composing.
    pub fn cancel(&mut self) {
        if self.state == CompositionState::Composing {
            self.preedit = Preedit::empty();
            self.state = CompositionState::Cancelled;
        }
    }

    /// Returns whether the composition is live
    /// ([`Composing`](CompositionState::Composing)).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.state, CompositionState::Composing)
    }

    /// Returns the byte range the composition occupies in the target buffer.
    #[must_use]
    pub const fn anchor(&self) -> TextRange {
        self.anchor
    }

    /// Returns the current preedit.
    #[must_use]
    pub const fn preedit(&self) -> &Preedit {
        &self.preedit
    }

    /// Returns the lifecycle state.
    #[must_use]
    pub const fn state(&self) -> CompositionState {
        self.state
    }

    /// Returns whether an external `edit` is valid against the composition (spec
    /// §5.12 "external edits validate the composition range").
    ///
    /// While [`Composing`](CompositionState::Composing) an external edit is
    /// invalid when it overlaps the composition [`anchor`](Self::anchor) — an
    /// edit inside the live composition range would corrupt it — so this returns
    /// `false` for an overlapping edit and `true` otherwise. When the composition
    /// is no longer composing there is nothing to protect, so it is always
    /// `true`.
    ///
    /// Overlap is tested with [`TextRange::intersects`], which treats
    /// endpoint-touching ranges (e.g. `0..2` and `2..4`) and empty ranges as
    /// non-overlapping.
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
