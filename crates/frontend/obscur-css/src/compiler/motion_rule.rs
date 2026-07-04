//! Selector-associated motion: [`CompiledMotionRule`] and [`CompiledAnimation`].
//!
//! A CSS `transition` or `animation` is a property *of the elements a rule
//! matches* — a `transition` says "when **my** property changes, ease it", and
//! an `animation` plays a named `@keyframes` on the elements the rule selects.
//! The flat [`transitions`](super::CompiledStylesheet::transitions) and
//! [`keyframes`](super::CompiledStylesheet::keyframes) outputs lose that
//! association, so the runtime cannot know *which* nodes a transition/animation
//! applies to.
//!
//! [`CompiledMotionRule`] restores it: it pairs a rule's authored
//! [`SelectorList`] with the transition templates and resolved keyframe
//! animations that rule declares, so the runtime can match the selector against
//! the retained tree and drive the motion on exactly the matched nodes (§4.17).

use obscur_motion::track::KeyframeTrack;
use obscur_style::property::PropertyId;
use obscur_style::selector::SelectorList;

use super::transition_compiler::CompiledTransition;

/// A resolved `animation` on a rule: the `@keyframes` tracks to play and the
/// duration to play them over.
///
/// Produced by resolving an `animation-name` reference against the stylesheet's
/// compiled `@keyframes`; an unknown name contributes no [`CompiledAnimation`].
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledAnimation {
    /// The per-property keyframe tracks (cloned from the referenced
    /// `@keyframes`), in first-seen property order.
    pub tracks: Vec<(PropertyId, KeyframeTrack)>,
    /// The duration to play the tracks over, in seconds.
    pub duration_secs: f32,
}

/// A style rule's motion, keyed by the selector list the rule matched.
///
/// Only rules that declare at least one `transition` or a resolvable `animation`
/// produce a [`CompiledMotionRule`]; a plain style rule contributes none.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledMotionRule {
    /// The authored selector list the transitions/animations apply to.
    pub selectors: SelectorList,
    /// The `transition` templates declared on the rule.
    pub transitions: Vec<CompiledTransition>,
    /// The resolved keyframe animations the rule plays.
    pub animations: Vec<CompiledAnimation>,
}
