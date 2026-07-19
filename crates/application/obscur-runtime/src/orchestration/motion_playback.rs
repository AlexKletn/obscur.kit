//! [`MotionPlayback`] — the playback controls an animation or transition starts
//! with (spec §4.17; css-animations-1 §3.6/§3.8/§3.9, css-transitions-1 §2.4).

use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};

/// How a started motion plays: its start delay, how many iterations it runs,
/// which direction iteration 1 takes, whether iterations alternate, and whether
/// the value fills the delay.
///
/// [`default`](Self::default) is every CSS initial value — no delay, exactly one
/// forward iteration, no alternation, no fill — which is what
/// [`UiRuntime::animate`](crate::UiRuntime::animate) starts a motion with.
///
/// The direction plus alternation pair models `animation-direction` exactly:
/// `alternate` is `Forward` + alternating and `alternate-reverse` is `Reverse` +
/// alternating, so **iteration 1** takes the named direction. That is the
/// spec's one-based parity — css-animations-1 §3.6: "For the purpose of
/// determining whether an iteration is even or odd, iterations start counting
/// from 1."
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionPlayback {
    /// The start delay, in seconds. Negative seeks that far into the play cycle
    /// at start.
    pub delay_secs: f32,
    /// How many iterations to run.
    pub repeat: RepeatCount,
    /// The direction iteration 1 plays in.
    pub direction: PlaybackDirection,
    /// Whether each iteration flips direction.
    pub alternates: bool,
    /// Whether the value applies during the start delay.
    pub fill: FillMode,
}

impl Default for MotionPlayback {
    fn default() -> Self {
        Self {
            delay_secs: 0.0,
            repeat: RepeatCount::Finite(1),
            direction: PlaybackDirection::Forward,
            alternates: false,
            fill: FillMode::None,
        }
    }
}

impl MotionPlayback {
    /// The default playback with only the start delay set — the shape a CSS
    /// `transition` needs, since `transition-delay` is its one playback control.
    #[must_use]
    pub fn delayed(delay_secs: f32) -> Self {
        Self {
            delay_secs,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_every_css_initial_value() {
        let playback = MotionPlayback::default();
        assert_eq!(playback.delay_secs, 0.0);
        assert_eq!(playback.repeat, RepeatCount::Finite(1));
        assert_eq!(playback.direction, PlaybackDirection::Forward);
        assert!(!playback.alternates);
        assert_eq!(playback.fill, FillMode::None);
    }

    #[test]
    fn delayed_sets_only_the_delay() {
        let playback = MotionPlayback::delayed(-0.5);
        assert_eq!(playback.delay_secs, -0.5);
        assert_eq!(playback.repeat, RepeatCount::Finite(1));
        assert_eq!(playback.fill, FillMode::None);
    }
}
