//! The typed IME composition lifecycle events (spec §5.12 IME).

use super::Preedit;

#[doc = include_str!("../../docs/api/ime/composition_event.doc/CompositionEvent.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionEvent {
    /// Begin a composition session.
    Start,
    /// Update the live composition with a new preedit.
    Update(Preedit),
    /// Commit the composition, finalizing the given text.
    Commit(Box<str>),
    /// Cancel the composition without committing.
    Cancel,
}

impl CompositionEvent {
    /// Returns whether this event ends the session: true for
    /// [`Commit`](Self::Commit) and [`Cancel`](Self::Cancel).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Commit(_) | Self::Cancel)
    }

    /// Returns the preedit carried by an [`Update`](Self::Update) event, or
    /// `None` for the others.
    #[must_use]
    pub const fn preedit(&self) -> Option<&Preedit> {
        match self {
            Self::Update(preedit) => Some(preedit),
            _ => None,
        }
    }

    /// Returns the finalized text carried by a [`Commit`](Self::Commit) event, or
    /// `None` for the others.
    #[must_use]
    pub fn committed_text(&self) -> Option<&str> {
        match self {
            Self::Commit(text) => Some(text),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_terminal_only_for_commit_and_cancel() {
        assert!(!CompositionEvent::Start.is_terminal());
        assert!(!CompositionEvent::Update(Preedit::new("x")).is_terminal());
        assert!(CompositionEvent::Commit(Box::from("done")).is_terminal());
        assert!(CompositionEvent::Cancel.is_terminal());
    }

    #[test]
    fn preedit_accessor() {
        let ev = CompositionEvent::Update(Preedit::new("ab"));
        assert_eq!(ev.preedit().map(Preedit::text), Some("ab"));
        assert!(CompositionEvent::Start.preedit().is_none());
        assert!(CompositionEvent::Cancel.preedit().is_none());
    }

    #[test]
    fn committed_text_accessor() {
        let ev = CompositionEvent::Commit(Box::from("final"));
        assert_eq!(ev.committed_text(), Some("final"));
        assert!(CompositionEvent::Start.committed_text().is_none());
        assert!(
            CompositionEvent::Update(Preedit::new("x"))
                .committed_text()
                .is_none()
        );
    }
}
