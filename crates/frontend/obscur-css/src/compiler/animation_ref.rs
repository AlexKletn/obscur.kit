use obscur_identity::InternedSymbol;
use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};

#[doc = include_str!("../../docs/api/compiler/animation_compiler.doc/AnimationRef.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationRef {
    /// The interned `@keyframes` name this rule plays.
    pub name: InternedSymbol,
    /// The animation duration, in seconds (`0.0` when unspecified).
    pub duration_secs: f32,
    /// `animation-delay`, in seconds. Negative values seek into the playback
    /// (css-animations-1 §3.8).
    pub delay_secs: f32,
    /// `animation-iteration-count`, as a repeat count.
    pub repeat: RepeatCount,
    /// The direction iteration 1 plays in — `reverse`/`alternate-reverse` start
    /// reversed (css-animations-1 §3.6).
    pub direction: PlaybackDirection,
    /// Whether each iteration flips direction (`alternate`/`alternate-reverse`).
    pub alternates: bool,
    /// `animation-fill-mode` (css-animations-1 §3.9).
    pub fill: FillMode,
}
