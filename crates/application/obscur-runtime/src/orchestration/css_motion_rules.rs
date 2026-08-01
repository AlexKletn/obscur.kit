#![doc = include_str!("css_motion_rules.doc/_module.md")]

use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};
use obscur_motion::track::{Easing, KeyframeTrack};
use obscur_style::property::PropertyId;
use obscur_style::selector::SelectorList;

#[doc = include_str!("css_motion_rules.doc/CssTransitionRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CssTransitionRule {
    #[doc = include_str!("css_motion_rules.doc/selectors.md")]
    selectors: SelectorList,
    #[doc = include_str!("css_motion_rules.doc/property.md")]
    property: PropertyId,
    #[doc = include_str!("css_motion_rules.doc/duration_secs.md")]
    duration_secs: f32,
    #[doc = include_str!("css_motion_rules.doc/easing.md")]
    easing: Easing,
    #[doc = include_str!("css_motion_rules.doc/delay_secs.md")]
    delay_secs: f32,
}

impl CssTransitionRule {
    #[doc = include_str!("css_motion_rules.doc/new.md")]
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

    #[doc = include_str!("css_motion_rules.doc/with_delay.md")]
    #[must_use]
    pub fn with_delay(mut self, delay_secs: f32) -> Self {
        self.delay_secs = delay_secs;
        self
    }

    #[doc = include_str!("css_motion_rules.doc/delay_secs_2.md")]
    #[must_use]
    pub fn delay_secs(&self) -> f32 {
        self.delay_secs
    }

    #[doc = include_str!("css_motion_rules.doc/selectors_2.md")]
    #[must_use]
    pub fn selectors(&self) -> &SelectorList {
        &self.selectors
    }

    #[doc = include_str!("css_motion_rules.doc/property_2.md")]
    #[must_use]
    pub fn property(&self) -> PropertyId {
        self.property
    }

    #[doc = include_str!("css_motion_rules.doc/duration_secs_2.md")]
    #[must_use]
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }

    #[doc = include_str!("css_motion_rules.doc/easing_2.md")]
    #[must_use]
    pub fn easing(&self) -> Easing {
        self.easing
    }
}

#[doc = include_str!("css_motion_rules.doc/CssKeyframeRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CssKeyframeRule {
    #[doc = include_str!("css_motion_rules.doc/selectors_3.md")]
    selectors: SelectorList,
    #[doc = include_str!("css_motion_rules.doc/tracks.md")]
    tracks: Vec<(PropertyId, KeyframeTrack)>,
    #[doc = include_str!("css_motion_rules.doc/duration_secs_3.md")]
    duration_secs: f32,
    #[doc = include_str!("css_motion_rules.doc/delay_secs_3.md")]
    delay_secs: f32,
    #[doc = include_str!("css_motion_rules.doc/repeat.md")]
    repeat: RepeatCount,
    #[doc = include_str!("css_motion_rules.doc/direction.md")]
    direction: PlaybackDirection,
    #[doc = include_str!("css_motion_rules.doc/alternates.md")]
    alternates: bool,
    #[doc = include_str!("css_motion_rules.doc/fill.md")]
    fill: FillMode,
}

impl CssKeyframeRule {
    #[doc = include_str!("css_motion_rules.doc/new_2.md")]
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

    #[doc = include_str!("css_motion_rules.doc/with_playback.md")]
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

    #[doc = include_str!("css_motion_rules.doc/delay_secs_4.md")]
    #[must_use]
    pub fn delay_secs(&self) -> f32 {
        self.delay_secs
    }

    #[doc = include_str!("css_motion_rules.doc/repeat_2.md")]
    #[must_use]
    pub fn repeat(&self) -> RepeatCount {
        self.repeat
    }

    #[doc = include_str!("css_motion_rules.doc/direction_2.md")]
    #[must_use]
    pub fn direction(&self) -> PlaybackDirection {
        self.direction
    }

    #[doc = include_str!("css_motion_rules.doc/alternates_2.md")]
    #[must_use]
    pub fn alternates(&self) -> bool {
        self.alternates
    }

    #[doc = include_str!("css_motion_rules.doc/fill_2.md")]
    #[must_use]
    pub fn fill(&self) -> FillMode {
        self.fill
    }

    #[doc = include_str!("css_motion_rules.doc/selectors_4.md")]
    #[must_use]
    pub fn selectors(&self) -> &SelectorList {
        &self.selectors
    }

    #[doc = include_str!("css_motion_rules.doc/tracks_2.md")]
    #[must_use]
    pub fn tracks(&self) -> &[(PropertyId, KeyframeTrack)] {
        &self.tracks
    }

    #[doc = include_str!("css_motion_rules.doc/duration_secs_4.md")]
    #[must_use]
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }
}

#[doc = include_str!("css_motion_rules.doc/CssMotionRules.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CssMotionRules {
    #[doc = include_str!("css_motion_rules.doc/transitions.md")]
    transitions: Vec<CssTransitionRule>,
    #[doc = include_str!("css_motion_rules.doc/keyframes.md")]
    keyframes: Vec<CssKeyframeRule>,
}

impl CssMotionRules {
    #[doc = include_str!("css_motion_rules.doc/new_3.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("css_motion_rules.doc/from_parts.md")]
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

    #[doc = include_str!("css_motion_rules.doc/transitions_2.md")]
    #[must_use]
    pub fn transitions(&self) -> &[CssTransitionRule] {
        &self.transitions
    }

    #[doc = include_str!("css_motion_rules.doc/keyframes_2.md")]
    #[must_use]
    pub fn keyframes(&self) -> &[CssKeyframeRule] {
        &self.keyframes
    }

    #[doc = include_str!("css_motion_rules.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.transitions.is_empty() && self.keyframes.is_empty()
    }
}
