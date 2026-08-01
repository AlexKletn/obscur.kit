#![doc = include_str!("event_phase.doc/_module.md")]

#[doc = include_str!("event_phase.doc/EventPhase.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventPhase {
    #[doc = include_str!("event_phase.doc/Capture.md")]
    Capture,
    #[doc = include_str!("event_phase.doc/Target.md")]
    Target,
    #[doc = include_str!("event_phase.doc/Bubble.md")]
    Bubble,
}

impl EventPhase {
    #[doc = include_str!("event_phase.doc/is_target.md")]
    #[must_use]
    pub const fn is_target(self) -> bool {
        matches!(self, Self::Target)
    }

    #[doc = include_str!("event_phase.doc/is_capture.md")]
    #[must_use]
    pub const fn is_capture(self) -> bool {
        matches!(self, Self::Capture)
    }

    #[doc = include_str!("event_phase.doc/is_bubble.md")]
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
