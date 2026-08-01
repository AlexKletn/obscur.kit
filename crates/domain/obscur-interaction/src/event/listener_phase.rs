#![doc = include_str!("listener_phase.doc/_module.md")]

use super::EventPhase;

#[doc = include_str!("listener_phase.doc/ListenerPhase.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ListenerPhase {
    #[doc = include_str!("listener_phase.doc/Capture.md")]
    Capture,
    #[doc = include_str!("listener_phase.doc/Bubble.md")]
    #[default]
    Bubble,
}

impl ListenerPhase {
    #[doc = include_str!("listener_phase.doc/runs_in.md")]
    #[must_use]
    pub const fn runs_in(self, phase: EventPhase) -> bool {
        match phase {
            EventPhase::Capture => matches!(self, Self::Capture),
            EventPhase::Target => true,
            EventPhase::Bubble => matches!(self, Self::Bubble),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_bubble() {
        assert_eq!(ListenerPhase::default(), ListenerPhase::Bubble);
    }

    #[test]
    fn capture_and_bubble_each_run_in_their_own_direction() {
        assert!(ListenerPhase::Capture.runs_in(EventPhase::Capture));
        assert!(!ListenerPhase::Capture.runs_in(EventPhase::Bubble));

        assert!(ListenerPhase::Bubble.runs_in(EventPhase::Bubble));
        assert!(!ListenerPhase::Bubble.runs_in(EventPhase::Capture));
    }

    /// The target is where the registration choice stops mattering.
    #[test]
    fn both_run_at_the_target() {
        assert!(ListenerPhase::Capture.runs_in(EventPhase::Target));
        assert!(ListenerPhase::Bubble.runs_in(EventPhase::Target));
    }
}
