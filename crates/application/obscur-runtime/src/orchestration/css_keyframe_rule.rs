use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};
use obscur_motion::track::KeyframeTrack;
use obscur_style::matching::{MediaEnvironment, RuleCondition};
use obscur_style::property::PropertyId;
use obscur_style::selector::SelectorList;

/// A CSS `@keyframes` animation bound to a selector: on the elements this
/// selector matches, each per-property [`KeyframeTrack`] plays over
/// `duration_secs` (spec §4.17).
#[derive(Debug, Clone, PartialEq)]
pub struct CssKeyframeRule {
    condition: Option<RuleCondition>,
    definition_group: Option<usize>,
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
            condition: None,
            definition_group: None,
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

    /// Restricts this animation rule to matching environments. Passing `None`
    /// makes the rule unconditional.
    #[must_use]
    pub fn with_condition(mut self, condition: Option<RuleCondition>) -> Self {
        self.condition = condition;
        self
    }

    /// Marks alternative definitions of one animation reference.
    #[must_use]
    pub fn with_definition_group(mut self, group: usize) -> Self {
        self.definition_group = Some(group);
        self
    }

    pub(crate) fn definition_group(&self) -> Option<usize> {
        self.definition_group
    }

    pub(crate) fn applies(&self, environment: &MediaEnvironment<'_>) -> bool {
        self.condition
            .as_ref()
            .is_none_or(|condition| condition.holds(environment))
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
