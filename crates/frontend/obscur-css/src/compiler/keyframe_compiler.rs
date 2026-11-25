#![doc = include_str!("../../docs/api/compiler/keyframe_compiler.doc/_module.md")]

use crate::parser::{KeyframeSelector, ParsedKeyframesRule, ParsedValue};

use obscur_geometry::AffineTransform;
use obscur_motion::track::{Keyframe, KeyframeTrack};
use obscur_motion::value::MotionValue;
use obscur_style::property::{Length, LengthExpr, PropertyId, PropertyValue, TransformFunction};

use super::CssSymbolInterner;
use super::longhand_catalog::property_id_from_name;
use super::property_compiler::narrow_f32;
use super::transform_lowering::lower_transform;

use super::CompiledKeyframes;

/// Lowers `@keyframes` rules into [`CompiledKeyframes`].
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyframeCompiler;

impl KeyframeCompiler {
    /// Creates a keyframe compiler.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/compiler/keyframe_compiler.doc/compile.md")]
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
                    if property == PropertyId::Gap {
                        if let [row] | [row, _] = declaration.value.as_slice() {
                            let column = declaration.value.get(1).unwrap_or(row);
                            for (axis, component) in
                                [(PropertyId::RowGap, row), (PropertyId::ColumnGap, column)]
                            {
                                if let Some(value) =
                                    lower_motion(axis, std::slice::from_ref(component))
                                {
                                    push_keyframe(
                                        &mut builders,
                                        axis,
                                        Keyframe::new(offset, value),
                                    );
                                }
                            }
                        }
                        continue;
                    }
                    let Some(value) = lower_motion(property, &declaration.value) else {
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
        CompiledKeyframes {
            condition: None,
            name,
            tracks,
        }
    }
}

/// Maps a keyframe selector to its normalized `[0, 1]` offset.
fn offset_of(selector: KeyframeSelector) -> f32 {
    match selector {
        KeyframeSelector::From => 0.0,
        KeyframeSelector::To => 1.0,
        KeyframeSelector::Percent(percent) => narrow_f32(percent / 100.0),
    }
}

/// Lowers a keyframe declaration to a scalar or a fixed transform matrix. A
/// transform that needs layout-dependent units is deferred by the property
/// compiler and cannot be sampled as a keyframe matrix here.
fn lower_motion(property: PropertyId, values: &[ParsedValue]) -> Option<MotionValue> {
    if property == PropertyId::Transform {
        return match lower_transform(values)? {
            PropertyValue::Transform(transform) => Some(MotionValue::transform(transform.matrix())),
            PropertyValue::TransformFunctions(functions) => lower_box_transform(&functions),
            _ => None,
        };
    }

    match values.first()? {
        ParsedValue::Number(n) => Some(MotionValue::scalar(narrow_f32(*n))),
        ParsedValue::Percentage(p) => Some(MotionValue::scalar(narrow_f32(p / 100.0))),
        ParsedValue::Dimension { value, unit } if unit.eq_ignore_ascii_case("px") => {
            Some(MotionValue::scalar(narrow_f32(*value)))
        }
        _ => None,
    }
}

/// Composes a transform list while retaining width and height dependent translation coefficients.
fn lower_box_transform(functions: &[TransformFunction]) -> Option<MotionValue> {
    let mut matrix = AffineTransform::IDENTITY;
    let mut relative = [0.0; 4];
    for function in functions {
        match function {
            TransformFunction::Matrix(coeffs) => {
                matrix = matrix.multiply(AffineTransform { coeffs: *coeffs });
            }
            TransformFunction::Translate(x, y) => {
                let (x_px, x_ratio) = affine_length(x)?;
                let (y_px, y_ratio) = affine_length(y)?;
                let [a, b, c, d, _, _] = matrix.coeffs;
                relative[0] += a * x_ratio;
                relative[1] += b * x_ratio;
                relative[2] += c * y_ratio;
                relative[3] += d * y_ratio;
                matrix = matrix.multiply(AffineTransform::translate(x_px, y_px));
            }
        }
    }
    Some(MotionValue::box_transform(matrix.coeffs, relative))
}

/// Lowers a linear length-percentage expression into pixel and percentage terms.
fn affine_length(expression: &LengthExpr) -> Option<(f32, f32)> {
    let (pixels, ratio) = match expression {
        LengthExpr::Leaf(Length::Px(px)) => (px.0, 0.0),
        LengthExpr::Leaf(Length::Percent(percent)) => (0.0, percent / 100.0),
        LengthExpr::Sum(a, b) | LengthExpr::Diff(a, b) => {
            let (a_px, a_ratio) = affine_length(a)?;
            let (b_px, b_ratio) = affine_length(b)?;
            let sign = if matches!(expression, LengthExpr::Sum(..)) {
                1.0
            } else {
                -1.0
            };
            (a_px + sign * b_px, a_ratio + sign * b_ratio)
        }
        LengthExpr::Product(term, scalar) | LengthExpr::Quotient(term, scalar) => {
            let (px, ratio) = affine_length(term)?;
            let factor = if matches!(expression, LengthExpr::Product(..)) {
                *scalar
            } else {
                scalar.recip()
            };
            (px * factor, ratio * factor)
        }
        _ => return None,
    };
    (pixels.is_finite() && ratio.is_finite()).then_some((pixels, ratio))
}

/// Appends `keyframe` to the track builder for `property`, creating it on first
/// use so property order matches first appearance.
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
    fn fixed_transform_keyframes_interpolate() {
        let (compiled, _) = keyframes(
            "@keyframes move { from { transform: none; } to { transform: translateX(10px); } }",
        );
        let sample = track_for(&compiled, PropertyId::Transform).sample(0.5);
        assert_eq!(
            sample,
            MotionValue::transform([1.0, 0.0, 0.0, 1.0, 5.0, 0.0])
        );
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
