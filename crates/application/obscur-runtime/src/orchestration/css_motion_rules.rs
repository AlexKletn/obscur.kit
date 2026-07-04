//! Runtime-facing CSS motion inputs: [`CssMotionRules`] and its rule types.
//!
//! The CSS frontend lowers `transition`/`@keyframes` into `obscur-motion` tracks
//! and keeps them associated with the selector the rule matched. This module is
//! the runtime's *domain-only* view of that association — [`SelectorList`],
//! [`PropertyId`], [`Easing`], [`KeyframeTrack`] and durations, with no
//! dependency on the CSS crate. A host builds a [`CssMotionRules`] from the
//! compiled stylesheet (see the facade's `counter_headless` example) and installs
//! it on the [`UiRuntime`](crate::UiRuntime) with
//! [`set_css_motion`](crate::UiRuntime::set_css_motion); each style frame the
//! runtime matches these rules against the retained tree and drives the motion
//! through the [`MotionScheduler`](obscur_motion::scheduler::MotionScheduler)
//! (spec §4.17).

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
        }
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
}

impl CssKeyframeRule {
    /// Creates a keyframe (animation) rule.
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
        }
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
