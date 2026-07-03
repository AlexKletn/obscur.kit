//! The [`GestureResolution`] — the outcome of resolving a gesture arena (spec
//! §6.16 mobile gestures).

use super::RecognizerId;

/// The outcome of resolving a [`GestureArena`](super::GestureArena) (spec
/// §6.16).
///
/// - [`Winner`](Self::Winner): a recognizer won the pointer sequence; the arena
///   cancels all other contenders.
/// - [`NoWinner`](Self::NoWinner): every contender failed — no gesture.
/// - [`Pending`](Self::Pending): contenders remain active and none has
///   resolved yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureResolution {
    /// The identified recognizer won; the others are cancelled.
    Winner(RecognizerId),
    /// All contenders resolved without recognition.
    NoWinner,
    /// The arena is still contending — no recognizer has resolved.
    Pending,
}

impl GestureResolution {
    /// Returns whether the arena has reached a final outcome
    /// ([`Winner`](Self::Winner) or [`NoWinner`](Self::NoWinner)).
    #[must_use]
    pub const fn is_resolved(self) -> bool {
        matches!(self, Self::Winner(_) | Self::NoWinner)
    }

    /// Returns the winning recognizer, if any.
    #[must_use]
    pub const fn winner(self) -> Option<RecognizerId> {
        match self {
            Self::Winner(id) => Some(id),
            Self::NoWinner | Self::Pending => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winner_is_resolved_and_reports_id() {
        let r = GestureResolution::Winner(RecognizerId::from_raw(5));
        assert!(r.is_resolved());
        assert_eq!(r.winner(), Some(RecognizerId::from_raw(5)));
    }

    #[test]
    fn no_winner_is_resolved_without_id() {
        let r = GestureResolution::NoWinner;
        assert!(r.is_resolved());
        assert_eq!(r.winner(), None);
    }

    #[test]
    fn pending_is_not_resolved() {
        let r = GestureResolution::Pending;
        assert!(!r.is_resolved());
        assert_eq!(r.winner(), None);
    }
}
