//! The typed IME composition lifecycle events (spec §5.12 IME).

use super::Preedit;

/// A typed IME composition lifecycle event (spec §5.12 start/update/commit/cancel).
///
/// These events map 1:1 onto [`Composition`](super::Composition)'s state
/// transitions: [`Start`](Self::Start) opens a session,
/// [`Update`](Self::Update) carries a new [`Preedit`], [`Commit`](Self::Commit)
/// finalizes with the committed text, and [`Cancel`](Self::Cancel) abandons it.
/// [`Commit`](Self::Commit) and [`Cancel`](Self::Cancel) are the terminal events
/// (see [`is_terminal`](Self::is_terminal)).
///
/// Per §5.12 duplicate keydown/text delivery must be avoided: the driver is
/// responsible for de-duplicating redundant events (for example a repeated
/// `Start` or a `Commit` after the session already ended), which the engine then
/// ignores.
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
