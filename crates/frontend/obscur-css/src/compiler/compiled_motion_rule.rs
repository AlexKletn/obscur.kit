#![doc = include_str!("../../docs/api/compiler/motion_rule.doc/_module.md")]

use obscur_style::matching::RuleCondition;
use obscur_style::selector::SelectorList;

use super::{CompiledAnimation, CompiledTransition};

#[doc = include_str!("../../docs/api/compiler/motion_rule.doc/CompiledMotionRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledMotionRule {
    /// Conditional group enclosing the style rule, if any.
    pub condition: Option<RuleCondition>,
    /// The authored selector list the transitions/animations apply to.
    pub selectors: SelectorList,
    /// The `transition` templates declared on the rule.
    pub transitions: Vec<CompiledTransition>,
    /// Whether the shorthand clears transitions from lower-priority declarations.
    pub transition_reset: bool,
    /// Whether the transition shorthand participates in the important cascade.
    pub transition_important: bool,
    /// The resolved keyframe animations the rule plays.
    pub animations: Vec<CompiledAnimation>,
}
