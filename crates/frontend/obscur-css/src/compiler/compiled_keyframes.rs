use obscur_identity::InternedSymbol;
use obscur_motion::track::KeyframeTrack;
use obscur_style::matching::RuleCondition;
use obscur_style::property::PropertyId;

#[doc = include_str!("../../docs/api/compiler/keyframe_compiler.doc/CompiledKeyframes.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledKeyframes {
    /// Conditional group enclosing the keyframe definition, if any.
    pub condition: Option<RuleCondition>,
    /// The interned animation name (the `@keyframes` identifier).
    pub name: InternedSymbol,
    /// The animated properties, each with its keyframe track.
    pub tracks: Vec<(PropertyId, KeyframeTrack)>,
}
