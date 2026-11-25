#![doc = include_str!("../../docs/api/orchestration/css_motion_rules.doc/_module.md")]

use super::{CssKeyframeRule, CssTransitionRule};

#[doc = include_str!("../../docs/api/orchestration/css_motion_rules.doc/CssMotionRules.md")]
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
