use obscur_identity::InternedSymbol;
use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};

use super::AnimationRef;

/// The playback controls an `animation` layer or the `animation-*` longhands
/// specify, independent of which `@keyframes` they apply to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct AnimationPlayback {
    /// `animation-delay`, in seconds.
    pub(super) delay_secs: f32,
    /// `animation-iteration-count`.
    pub(super) repeat: RepeatCount,
    /// The direction iteration 1 plays in.
    pub(super) direction: PlaybackDirection,
    /// Whether iterations alternate direction.
    pub(super) alternates: bool,
    /// `animation-fill-mode`.
    pub(super) fill: FillMode,
}

impl Default for AnimationPlayback {
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

impl AnimationPlayback {
    pub(super) fn bind(self, name: InternedSymbol, duration_secs: f32) -> AnimationRef {
        AnimationRef {
            name,
            duration_secs,
            delay_secs: self.delay_secs,
            repeat: self.repeat,
            direction: self.direction,
            alternates: self.alternates,
            fill: self.fill,
        }
    }
}
