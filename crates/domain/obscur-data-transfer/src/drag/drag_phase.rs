#[doc = include_str!("../../docs/api/drag/drag_session.doc/DragPhase.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragPhase {
    /// The drag has begun but the pointer is not over a drop target (spec
    /// §6.21).
    Started,
    /// The pointer is currently over an accepting drop target (spec §6.21).
    Over,
    /// A valid drop committed — terminal (spec §6.21).
    Dropped,
    /// The drag was cancelled — terminal (spec §6.21).
    Cancelled,
}

impl DragPhase {
    /// Returns whether this is a terminal phase —
    /// [`Dropped`](DragPhase::Dropped) or [`Cancelled`](DragPhase::Cancelled)
    /// (spec §6.21).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Dropped | Self::Cancelled)
    }

    /// Returns whether this is an active phase —
    /// [`Started`](DragPhase::Started) or [`Over`](DragPhase::Over) (spec
    /// §6.21).
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Started | Self::Over)
    }
}

#[cfg(test)]
mod tests {
    use super::DragPhase;

    #[test]
    fn phase_predicates() {
        assert!(DragPhase::Started.is_active());
        assert!(DragPhase::Over.is_active());
        assert!(!DragPhase::Dropped.is_active());
        assert!(DragPhase::Dropped.is_terminal());
        assert!(DragPhase::Cancelled.is_terminal());
        assert!(!DragPhase::Started.is_terminal());
    }
}
