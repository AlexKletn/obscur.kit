use obscur_style::matching::RuleCondition;
use obscur_style::selector::SelectorList;

use super::{AnimationRef, CompiledTransition};

/// A style rule's motion before `@keyframes` names are resolved to tracks: the
/// rule's selector list, its transition templates, and its (still-by-name)
/// animation references. Resolved into a [`CompiledMotionRule`](super::CompiledMotionRule) in
/// [`StylesheetCompiler::compile`](super::StylesheetCompiler::compile), where every `@keyframes` rule is known.
pub(super) struct PendingMotionRule {
    pub(super) condition: Option<RuleCondition>,
    pub(super) selectors: SelectorList,
    pub(super) transitions: Vec<CompiledTransition>,
    pub(super) transition_reset: bool,
    pub(super) transition_important: bool,
    pub(super) animations: Vec<AnimationRef>,
}
