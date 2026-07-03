//! The DOM-like propagation phase an event is in as it routes (spec §4.7).

/// The phase of propagation an event occupies as it travels the logical tree
/// (spec §4.7 hit testing → event routing).
///
/// Mirrors the classic three-phase model: an event first descends from the root
/// toward the target ([`Capture`](Self::Capture)), then fires at the target
/// itself ([`Target`](Self::Target)), then ascends back toward the root
/// ([`Bubble`](Self::Bubble)). The
/// [`EventDispatcher`](crate::event::EventDispatcher) stamps each step of the
/// computed path with one of these phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventPhase {
    /// Descending from the root toward (but not including) the target.
    Capture,
    /// Firing at the hit-test target itself.
    Target,
    /// Ascending from the target back toward the root.
    Bubble,
}

impl EventPhase {
    /// Returns whether this is the [`Target`](Self::Target) phase.
    #[must_use]
    pub const fn is_target(self) -> bool {
        matches!(self, Self::Target)
    }

    /// Returns whether this is the [`Capture`](Self::Capture) phase.
    #[must_use]
    pub const fn is_capture(self) -> bool {
        matches!(self, Self::Capture)
    }

    /// Returns whether this is the [`Bubble`](Self::Bubble) phase.
    #[must_use]
    pub const fn is_bubble(self) -> bool {
        matches!(self, Self::Bubble)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_predicates() {
        assert!(EventPhase::Capture.is_capture());
        assert!(!EventPhase::Capture.is_target());
        assert!(!EventPhase::Capture.is_bubble());

        assert!(EventPhase::Target.is_target());
        assert!(!EventPhase::Target.is_capture());
        assert!(!EventPhase::Target.is_bubble());

        assert!(EventPhase::Bubble.is_bubble());
        assert!(!EventPhase::Bubble.is_target());
        assert!(!EventPhase::Bubble.is_capture());
    }
}
