#![doc = include_str!("motion_playback.doc/_module.md")]

use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};

#[doc = include_str!("motion_playback.doc/MotionPlayback.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionPlayback {
    #[doc = include_str!("motion_playback.doc/delay_secs.md")]
    pub delay_secs: f32,
    #[doc = include_str!("motion_playback.doc/repeat.md")]
    pub repeat: RepeatCount,
    #[doc = include_str!("motion_playback.doc/direction.md")]
    pub direction: PlaybackDirection,
    #[doc = include_str!("motion_playback.doc/alternates.md")]
    pub alternates: bool,
    #[doc = include_str!("motion_playback.doc/fill.md")]
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
    #[doc = include_str!("motion_playback.doc/delayed.md")]
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
