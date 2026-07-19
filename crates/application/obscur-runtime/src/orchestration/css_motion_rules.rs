//! Runtime-facing CSS motion inputs: [`CssMotionRules`] and its rule types.
//!
//! The CSS frontend lowers `transition`/`@keyframes` into `obscur-motion` tracks
//! and keeps them associated with the selector the rule matched. This module is
//! the runtime's *domain-only* view of that association — [`SelectorList`],
//! [`PropertyId`], [`Easing`], [`KeyframeTrack`] and durations, with no
//! dependency on the CSS crate. Building one from a compiled stylesheet and
//! installing it with [`set_css_motion`](crate::UiRuntime::set_css_motion) is the
//! facade's job — `obscur_kit::bootstrap::CssPipeline::install` does it, so a
//! host that installs CSS through the kit never touches these types. Each style
//! frame the
//! runtime matches these rules against the retained tree and drives the motion
//! through the [`MotionScheduler`](obscur_motion::scheduler::MotionScheduler)
//! (spec §4.17).

use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};
use obscur_motion::track::{Easing, KeyframeTrack};
use obscur_style::property::PropertyId;
use obscur_style::selector::SelectorList;

/// A CSS `transition` bound to a selector: on the elements this selector matches,
/// a change to `property`'s computed value is eased over `duration_secs` using
/// `easing` (spec §4.17).
#[derive(Debug, Clone, PartialEq)]
pub struct CssTransitionRule {
    /// The selector list whose matched elements carry this transition.
    selectors: SelectorList,
    /// The property whose change triggers the transition.
    property: PropertyId,
    /// The transition duration, in seconds.
    duration_secs: f32,
    /// The easing curve applied over the transition.
    easing: Easing,
    /// `transition-delay`, in seconds. A negative delay makes the transition
    /// start part-way through its play cycle (css-transitions-1 §2.4).
    delay_secs: f32,
}

impl CssTransitionRule {
    /// Creates a transition rule.
    #[must_use]
    pub fn new(
        selectors: SelectorList,
        property: PropertyId,
        duration_secs: f32,
        easing: Easing,
    ) -> Self {
        Self {
            selectors,
            property,
            duration_secs,
            easing,
            delay_secs: 0.0,
        }
    }

    /// Sets `transition-delay` in seconds, returning the updated rule.
    ///
    /// A positive delay holds the transition at its start value for that long; a
    /// negative one makes it begin part-way through its play cycle
    /// (css-transitions-1 §2.4).
    #[must_use]
    pub fn with_delay(mut self, delay_secs: f32) -> Self {
        self.delay_secs = delay_secs;
        self
    }

    /// Returns `transition-delay`, in seconds.
    #[must_use]
    pub fn delay_secs(&self) -> f32 {
        self.delay_secs
    }

    /// Returns the selector list this transition applies to.
    #[must_use]
    pub fn selectors(&self) -> &SelectorList {
        &self.selectors
    }

    /// Returns the transitioned property.
    #[must_use]
    pub fn property(&self) -> PropertyId {
        self.property
    }

    /// Returns the transition duration, in seconds.
    #[must_use]
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }

    /// Returns the easing curve.
    #[must_use]
    pub fn easing(&self) -> Easing {
        self.easing
    }
}

/// A CSS `@keyframes` animation bound to a selector: on the elements this
/// selector matches, each per-property [`KeyframeTrack`] plays over
/// `duration_secs` (spec §4.17).
#[derive(Debug, Clone, PartialEq)]
pub struct CssKeyframeRule {
    /// The selector list whose matched elements play this animation.
    selectors: SelectorList,
    /// The per-property keyframe tracks to play.
    tracks: Vec<(PropertyId, KeyframeTrack)>,
    /// The duration to play the tracks over, in seconds.
    duration_secs: f32,
    /// `animation-delay`, in seconds (negative seeks into the playback).
    delay_secs: f32,
    /// `animation-iteration-count`.
    repeat: RepeatCount,
    /// The direction iteration 1 plays in — `reverse` and `alternate-reverse`
    /// start reversed (css-animations-1 §3.6).
    direction: PlaybackDirection,
    /// Whether each iteration flips direction (`alternate`/`alternate-reverse`).
    alternates: bool,
    /// `animation-fill-mode` (css-animations-1 §3.9).
    fill: FillMode,
}

impl CssKeyframeRule {
    /// Creates a keyframe (animation) rule with every playback control at its
    /// CSS initial value: no delay, one iteration, `normal` direction, `none`
    /// fill. Use [`with_playback`](Self::with_playback) to set them.
    #[must_use]
    pub fn new(
        selectors: SelectorList,
        tracks: Vec<(PropertyId, KeyframeTrack)>,
        duration_secs: f32,
    ) -> Self {
        Self {
            selectors,
            tracks,
            duration_secs,
            delay_secs: 0.0,
            repeat: RepeatCount::Finite(1),
            direction: PlaybackDirection::Forward,
            alternates: false,
            fill: FillMode::None,
        }
    }

    /// Sets the `animation-delay`, `animation-iteration-count`,
    /// `animation-direction` and `animation-fill-mode` controls, returning the
    /// updated rule.
    #[must_use]
    pub fn with_playback(
        mut self,
        delay_secs: f32,
        repeat: RepeatCount,
        direction: PlaybackDirection,
        alternates: bool,
        fill: FillMode,
    ) -> Self {
        self.delay_secs = delay_secs;
        self.repeat = repeat;
        self.direction = direction;
        self.alternates = alternates;
        self.fill = fill;
        self
    }

    /// Returns `animation-delay`, in seconds.
    #[must_use]
    pub fn delay_secs(&self) -> f32 {
        self.delay_secs
    }

    /// Returns `animation-iteration-count`.
    #[must_use]
    pub fn repeat(&self) -> RepeatCount {
        self.repeat
    }

    /// Returns the direction iteration 1 plays in.
    #[must_use]
    pub fn direction(&self) -> PlaybackDirection {
        self.direction
    }

    /// Returns whether iterations alternate direction.
    #[must_use]
    pub fn alternates(&self) -> bool {
        self.alternates
    }

    /// Returns `animation-fill-mode`.
    #[must_use]
    pub fn fill(&self) -> FillMode {
        self.fill
    }

    /// Returns the selector list this animation applies to.
    #[must_use]
    pub fn selectors(&self) -> &SelectorList {
        &self.selectors
    }

    /// Returns the per-property keyframe tracks.
    #[must_use]
    pub fn tracks(&self) -> &[(PropertyId, KeyframeTrack)] {
        &self.tracks
    }

    /// Returns the animation duration, in seconds.
    #[must_use]
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }
}

/// The set of CSS-authored transitions and animations the runtime drives, keyed
/// by selector (spec §4.17).
///
/// Built by a host from the compiled stylesheet and installed on the runtime;
/// empty by default (a runtime with no CSS motion drives nothing here).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CssMotionRules {
    /// The selector-bound transitions.
    transitions: Vec<CssTransitionRule>,
    /// The selector-bound keyframe animations.
    keyframes: Vec<CssKeyframeRule>,
}

impl CssMotionRules {
    /// Creates an empty rule set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a rule set from its transitions and keyframe animations.
    #[must_use]
    pub fn from_parts(
        transitions: Vec<CssTransitionRule>,
        keyframes: Vec<CssKeyframeRule>,
    ) -> Self {
        Self {
            transitions,
            keyframes,
        }
    }

    /// Returns the transition rules.
    #[must_use]
    pub fn transitions(&self) -> &[CssTransitionRule] {
        &self.transitions
    }

    /// Returns the keyframe-animation rules.
    #[must_use]
    pub fn keyframes(&self) -> &[CssKeyframeRule] {
        &self.keyframes
    }

    /// Returns whether the rule set drives no motion at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.transitions.is_empty() && self.keyframes.is_empty()
    }
}
