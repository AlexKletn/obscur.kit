#![doc = include_str!("keyframe_compiler.doc/_module.md")]

use crate::parser::{KeyframeSelector, ParsedKeyframesRule, ParsedValue};

use obscur_identity::InternedSymbol;
use obscur_motion::track::{Keyframe, KeyframeTrack};
use obscur_motion::value::MotionValue;
use obscur_style::property::PropertyId;

use super::longhand_catalog::property_id_from_name;
use super::property_compiler::narrow_f32;
use super::stylesheet_compiler::CssSymbolInterner;

#[doc = include_str!("keyframe_compiler.doc/CompiledKeyframes.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledKeyframes {
    #[doc = include_str!("keyframe_compiler.doc/name.md")]
    pub name: InternedSymbol,
    #[doc = include_str!("keyframe_compiler.doc/tracks.md")]
    pub tracks: Vec<(PropertyId, KeyframeTrack)>,
}

#[doc = include_str!("keyframe_compiler.doc/KeyframeCompiler.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyframeCompiler;

impl KeyframeCompiler {
    #[doc = include_str!("keyframe_compiler.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("keyframe_compiler.doc/compile.md")]
    #[must_use]
    pub fn compile(
        &self,
        rule: &ParsedKeyframesRule,
        interner: &mut CssSymbolInterner,
    ) -> CompiledKeyframes {
        let name = interner.intern(&rule.name);
        // First-seen property order, each accumulating its keyframes.
        let mut builders: Vec<(PropertyId, Vec<Keyframe>)> = Vec::new();
        for frame in &rule.keyframes {
            for selector in &frame.selectors {
                let offset = offset_of(*selector);
                for declaration in &frame.declarations {
                    let Some(property) = property_id_from_name(&declaration.name) else {
                        continue;
                    };
                    let Some(value) = lower_motion(&declaration.value) else {
                        continue;
                    };
                    push_keyframe(&mut builders, property, Keyframe::new(offset, value));
                }
            }
        }
        let tracks = builders
            .into_iter()
            .map(|(property, keyframes)| (property, KeyframeTrack::from_keyframes(keyframes)))
            .collect();
        CompiledKeyframes { name, tracks }
    }
}

#[doc = include_str!("keyframe_compiler.doc/offset_of.md")]
fn offset_of(selector: KeyframeSelector) -> f32 {
    match selector {
        KeyframeSelector::From => 0.0,
        KeyframeSelector::To => 1.0,
        KeyframeSelector::Percent(percent) => narrow_f32(percent / 100.0),
    }
}

#[doc = include_str!("keyframe_compiler.doc/lower_motion.md")]
fn lower_motion(values: &[ParsedValue]) -> Option<MotionValue> {
    match values.first()? {
        ParsedValue::Number(n) => Some(MotionValue::scalar(narrow_f32(*n))),
        ParsedValue::Percentage(p) => Some(MotionValue::scalar(narrow_f32(p / 100.0))),
        ParsedValue::Dimension { value, unit } if unit.eq_ignore_ascii_case("px") => {
            Some(MotionValue::scalar(narrow_f32(*value)))
        }
        _ => None,
    }
}

#[doc = include_str!("keyframe_compiler.doc/push_keyframe.md")]
fn push_keyframe(
    builders: &mut Vec<(PropertyId, Vec<Keyframe>)>,
    property: PropertyId,
    keyframe: Keyframe,
) {
    if let Some((_, keyframes)) = builders.iter_mut().find(|(id, _)| *id == property) {
        keyframes.push(keyframe);
    } else {
        builders.push((property, vec![keyframe]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedRule, StylesheetParser};
    use crate::source::CssSource;
    use obscur_motion::track::MotionTrack;

    fn keyframes(text: &str) -> (CompiledKeyframes, CssSymbolInterner) {
        let src = CssSource::anonymous(text);
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Keyframes(rule) = &sheet.rules[0] else {
            panic!("expected keyframes rule");
        };
        let mut interner = CssSymbolInterner::new();
        let compiled = KeyframeCompiler::new().compile(rule, &mut interner);
        (compiled, interner)
    }

    fn track_for(compiled: &CompiledKeyframes, property: PropertyId) -> &KeyframeTrack {
        &compiled
            .tracks
            .iter()
            .find(|(id, _)| *id == property)
            .expect("expected a track for the property")
            .1
    }

    #[test]
    fn interns_the_animation_name() {
        let (compiled, interner) =
            keyframes("@keyframes spin { from { opacity: 0 } to { opacity: 1 } }");
        assert_eq!(interner.resolve(compiled.name), Some("spin"));
    }

    #[test]
    fn from_and_to_become_zero_and_one() {
        let (compiled, _) = keyframes("@keyframes fade { from { opacity: 0 } to { opacity: 1 } }");
        let track = track_for(&compiled, PropertyId::Opacity);
        let offsets: Vec<f32> = track.keyframes().iter().map(Keyframe::offset).collect();
        assert_eq!(offsets, vec![0.0, 1.0]);
    }

    #[test]
    fn percentage_selectors_become_fractional_offsets() {
        let (compiled, _) =
            keyframes("@keyframes k { 0% { opacity: 0 } 50% { opacity: 1 } 100% { opacity: 0 } }");
        let track = track_for(&compiled, PropertyId::Opacity);
        let offsets: Vec<f32> = track.keyframes().iter().map(Keyframe::offset).collect();
        assert_eq!(offsets, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn opacity_values_lower_to_scalars() {
        let (compiled, _) = keyframes("@keyframes fade { from { opacity: 0 } to { opacity: 1 } }");
        let track = track_for(&compiled, PropertyId::Opacity);
        assert_eq!(track.sample(0.0), MotionValue::scalar(0.0));
        assert_eq!(track.sample(1.0), MotionValue::scalar(1.0));
        assert_eq!(track.sample(0.5), MotionValue::scalar(0.5));
    }

    #[test]
    fn px_lengths_lower_to_scalars() {
        let (compiled, _) =
            keyframes("@keyframes grow { from { width: 0px } to { width: 100px } }");
        let track = track_for(&compiled, PropertyId::Width);
        assert_eq!(track.sample(0.0), MotionValue::scalar(0.0));
        assert_eq!(track.sample(1.0), MotionValue::scalar(100.0));
    }

    #[test]
    fn one_track_per_property() {
        let (compiled, _) = keyframes(
            "@keyframes multi { from { opacity: 0; width: 0px } to { opacity: 1; width: 10px } }",
        );
        assert_eq!(compiled.tracks.len(), 2);
    }

    #[test]
    fn shared_offset_across_grouped_selectors() {
        // `0%, 100%` applies the same keyframe at both ends.
        let (compiled, _) =
            keyframes("@keyframes pulse { 0%, 100% { opacity: 1 } 50% { opacity: 0 } }");
        let track = track_for(&compiled, PropertyId::Opacity);
        assert_eq!(track.len(), 3);
        assert_eq!(track.sample(0.0), MotionValue::scalar(1.0));
        assert_eq!(track.sample(1.0), MotionValue::scalar(1.0));
        assert_eq!(track.sample(0.5), MotionValue::scalar(0.0));
    }

    #[test]
    fn non_animatable_values_are_skipped() {
        // `display: block` has no scalar motion value, so it produces no track.
        let (compiled, _) =
            keyframes("@keyframes k { from { display: block } to { display: flex } }");
        assert!(compiled.tracks.is_empty());
    }

    #[test]
    fn unknown_properties_are_skipped() {
        let (compiled, _) = keyframes("@keyframes k { from { margin: 0px } to { margin: 10px } }");
        assert!(compiled.tracks.is_empty());
    }

    #[test]
    fn keyframes_are_sorted_by_offset() {
        let (compiled, _) = keyframes(
            "@keyframes k { 100% { opacity: 1 } 0% { opacity: 0 } 50% { opacity: 0.5 } }",
        );
        let track = track_for(&compiled, PropertyId::Opacity);
        let offsets: Vec<f32> = track.keyframes().iter().map(Keyframe::offset).collect();
        assert_eq!(offsets, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn empty_keyframes_rule_has_no_tracks() {
        let (compiled, _) = keyframes("@keyframes empty { }");
        assert!(compiled.tracks.is_empty());
    }
}
