//! [`MotionPlayback`] — the playback controls an animation or transition starts
//! with (spec §4.17; css-animations-1 §3.6/§3.8/§3.9, css-transitions-1 §2.4).

use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};

#[doc = include_str!("../../docs/api/orchestration/motion_playback.doc/MotionPlayback.md")]
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
