//! The [`TransitionInterruption`] redirect policy for a transition (spec §4.17
//! "interruption").

use crate::track::KeyframeTrack;
use crate::value::MotionValue;

#[doc = include_str!("../../docs/api/transition/transition_interruption.doc/TransitionInterruption.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionInterruption {
    /// Discard the current transition and start the new one from its beginning
    /// (spec §4.17). The element may visibly jump to the new start value.
    Restart,
    /// Turn the current transition around so it plays back toward its origin
    /// (spec §4.17). Continues from the present value — used to *undo* an
    /// in-flight enter/exit.
    Reverse,
    /// Start the new transition *from* the element's current mid-animation value
    /// (spec §4.17), producing a seamless redirect with no jump. The smoothest
    /// option and the default.
    Blend,
    /// Let the current transition finish and drop the new one (spec §4.17).
    Ignore,
}

impl TransitionInterruption {
    #[doc = include_str!("../../docs/api/transition/transition_interruption.doc/preserves_current_value.md")]
    #[must_use]
    pub const fn preserves_current_value(&self) -> bool {
        matches!(self, Self::Blend | Self::Reverse)
    }

    #[doc = include_str!("../../docs/api/transition/transition_interruption.doc/resolve_track.md")]
    #[must_use]
    pub fn resolve_track(
        self,
        target: &KeyframeTrack,
        current: MotionValue,
        origin: MotionValue,
    ) -> Option<KeyframeTrack> {
        match self {
            Self::Restart => Some(target.clone()),
            Self::Blend => Some(target.with_start_value(current)),
            Self::Reverse => Some(KeyframeTrack::from_endpoints(current, origin)),
            Self::Ignore => None,
        }
    }
}

impl Default for TransitionInterruption {
    /// [`Blend`](Self::Blend) — the smoothest UI default: an incoming transition
    /// redirects seamlessly from the element's current value (spec §4.17).
    fn default() -> Self {
        Self::Blend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::MotionTrack;

    #[test]
    fn default_is_blend() {
        assert_eq!(
            TransitionInterruption::default(),
            TransitionInterruption::Blend
        );
    }

    #[test]
    fn preserves_current_value_only_for_blend_and_reverse() {
        assert!(TransitionInterruption::Blend.preserves_current_value());
        assert!(TransitionInterruption::Reverse.preserves_current_value());
        assert!(!TransitionInterruption::Restart.preserves_current_value());
        assert!(!TransitionInterruption::Ignore.preserves_current_value());
    }

    /// A 0 -> 100 track whose *current* mid-flight value is 30 and whose origin
    /// is 0. Superseding it targets 0 -> 200.
    fn target() -> KeyframeTrack {
        KeyframeTrack::from_endpoints(MotionValue::scalar(0.0), MotionValue::scalar(200.0))
    }

    #[test]
    fn blend_starts_from_the_current_value_not_a_snap() {
        let resolved = TransitionInterruption::Blend
            .resolve_track(
                &target(),
                MotionValue::scalar(30.0),
                MotionValue::scalar(0.0),
            )
            .expect("Blend resolves a track");
        // Seamless: the new track begins at the current value (30), not 0.
        assert_eq!(resolved.sample(0.0), MotionValue::scalar(30.0));
        assert_eq!(resolved.sample(1.0), MotionValue::scalar(200.0));
    }

    #[test]
    fn restart_uses_the_incoming_track_verbatim() {
        let resolved = TransitionInterruption::Restart
            .resolve_track(
                &target(),
                MotionValue::scalar(30.0),
                MotionValue::scalar(0.0),
            )
            .expect("Restart resolves a track");
        assert_eq!(resolved, target());
        assert_eq!(resolved.sample(0.0), MotionValue::scalar(0.0));
    }

    #[test]
    fn reverse_plays_from_current_back_to_origin() {
        let resolved = TransitionInterruption::Reverse
            .resolve_track(
                &target(),
                MotionValue::scalar(30.0),
                MotionValue::scalar(0.0),
            )
            .expect("Reverse resolves a track");
        assert_eq!(resolved.sample(0.0), MotionValue::scalar(30.0));
        assert_eq!(resolved.sample(1.0), MotionValue::scalar(0.0));
    }

    #[test]
    fn ignore_resolves_no_track() {
        assert_eq!(
            TransitionInterruption::Ignore.resolve_track(
                &target(),
                MotionValue::scalar(30.0),
                MotionValue::scalar(0.0)
            ),
            None
        );
    }
}
