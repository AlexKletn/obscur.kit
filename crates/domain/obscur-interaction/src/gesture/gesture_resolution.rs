#![doc = include_str!("gesture_resolution.doc/_module.md")]

use super::RecognizerId;

#[doc = include_str!("gesture_resolution.doc/GestureResolution.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureResolution {
    #[doc = include_str!("gesture_resolution.doc/Winner.md")]
    Winner(RecognizerId),
    #[doc = include_str!("gesture_resolution.doc/NoWinner.md")]
    NoWinner,
    #[doc = include_str!("gesture_resolution.doc/Pending.md")]
    Pending,
}

impl GestureResolution {
    #[doc = include_str!("gesture_resolution.doc/is_resolved.md")]
    #[must_use]
    pub const fn is_resolved(self) -> bool {
        matches!(self, Self::Winner(_) | Self::NoWinner)
    }

    #[doc = include_str!("gesture_resolution.doc/winner.md")]
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
