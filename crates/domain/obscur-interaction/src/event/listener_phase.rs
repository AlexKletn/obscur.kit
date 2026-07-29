//! Which of §2.3's propagation phases a listener asked to be run in.

use super::EventPhase;

/// The phase a listener *registers* for (spec §2.3).
///
/// §2.3's propagation is *"1. capture; 2. target; 3. bubble; 4. default action"*,
/// but only two of those are a **registration** choice: a listener asks to be run
/// on the way down or on the way up. [`EventPhase::Target`] is not a choice —
/// every listener attached to the target node runs when the event reaches it,
/// whichever direction it registered for — and the default action is not a
/// listener at all. Hence two variants, not four.
///
/// # Why the default is `Bubble`
///
/// Capture is the unusual request: it lets an ancestor see an event *before* the
/// node the user actually interacted with, which is how an overlay or a
/// gesture arbiter takes an interaction away from its target. Bubble is what a
/// handler ordinarily wants — "tell me after the thing that was hit has had its
/// say" — so it is the default, and capture must be asked for explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ListenerPhase {
    /// Run while the event descends from the root toward the target, *before*
    /// the target itself is reached.
    Capture,
    /// Run while the event ascends from the target back toward the root.
    #[default]
    Bubble,
}

impl ListenerPhase {
    /// Returns whether a listener registered for this phase runs during `phase`.
    ///
    /// At [`EventPhase::Target`] the answer is `true` for **both** variants: the
    /// event has arrived at the node the listener is attached to, so the
    /// direction it registered for no longer distinguishes anything. This mirrors
    /// the DOM, where target-phase listeners run in registration order regardless
    /// of their capture flag.
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
