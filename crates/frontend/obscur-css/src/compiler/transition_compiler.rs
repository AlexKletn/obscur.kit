#![doc = include_str!("../../docs/api/compiler/transition_compiler.doc/_module.md")]

use crate::error::{UnsupportedCssFeature, UnsupportedFeatureKind};
use crate::parser::{ParsedDeclaration, ParsedValue};

use obscur_motion::track::Easing;
use obscur_style::property::PropertyId;

use super::longhand_catalog::property_id_from_name;
use super::property_compiler::narrow_f32;

use super::CompiledTransition;

/// Lowers `transition` declarations into [`CompiledTransition`] templates.
#[derive(Debug, Clone, Copy, Default)]
pub struct TransitionCompiler;

impl TransitionCompiler {
    /// Creates a transition compiler.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/compiler/transition_compiler.doc/compile.md")]
    #[must_use]
    pub fn compile(
        &self,
        decl: &ParsedDeclaration,
        unsupported: &mut Vec<UnsupportedCssFeature>,
    ) -> Vec<CompiledTransition> {
        let mut transitions = Vec::new();
        for layer in split_layers(&decl.value) {
            if let Some(transition) = compile_layer(layer, decl.span, unsupported) {
                if transition.property == PropertyId::Gap {
                    transitions.extend([PropertyId::RowGap, PropertyId::ColumnGap].map(
                        |property| CompiledTransition {
                            property,
                            ..transition
                        },
                    ));
                } else {
                    transitions.push(transition);
                }
            }
        }
        transitions
    }
}

/// Splits a component-value list into comma-separated layers.
fn split_layers(values: &[ParsedValue]) -> Vec<&[ParsedValue]> {
    values
        .split(|value| matches!(value, ParsedValue::Comma))
        .filter(|layer| !layer.is_empty())
        .collect()
}

/// Compiles one transition layer, or [`None`] when no property resolves.
fn compile_layer(
    layer: &[ParsedValue],
    span: crate::source::SourceSpan,
    unsupported: &mut Vec<UnsupportedCssFeature>,
) -> Option<CompiledTransition> {
    let mut property: Option<PropertyId> = None;
    let mut times: Vec<f32> = Vec::new();
    let mut easing: Option<Easing> = None;
    for component in layer {
        match component {
            ParsedValue::Keyword(keyword) => {
                if let Some(id) = property_id_from_name(keyword) {
                    property = Some(id);
                } else if let Some(curve) = easing_from_keyword(keyword) {
                    easing = Some(curve);
                }
                // Other keywords (for example `all`) leave the property unset.
            }
            ParsedValue::Dimension { value, unit } => {
                if unit.eq_ignore_ascii_case("s") {
                    times.push(narrow_f32(*value));
                } else if unit.eq_ignore_ascii_case("ms") {
                    times.push(narrow_f32(*value) / 1000.0);
                }
            }
            ParsedValue::Number(n) if *n == 0.0 => times.push(0.0),
            ParsedValue::Function { name, .. }
                if name.eq_ignore_ascii_case("cubic-bezier")
                    || name.eq_ignore_ascii_case("steps") =>
            {
                easing = Some(Easing::Linear);
                unsupported.push(UnsupportedCssFeature::new(
                    UnsupportedFeatureKind::Function,
                    &**name,
                    span,
                ));
            }
            _ => {}
        }
    }
    Some(CompiledTransition {
        property: property?,
        duration_secs: times.first().copied().unwrap_or(0.0),
        delay_secs: times.get(1).copied().unwrap_or(0.0),
        easing: easing.unwrap_or_default(),
    })
}

/// Maps a CSS timing-function keyword to an [`Easing`], or [`None`].
fn easing_from_keyword(keyword: &str) -> Option<Easing> {
    if keyword.eq_ignore_ascii_case("linear") {
        Some(Easing::Linear)
    } else if keyword.eq_ignore_ascii_case("ease-in") {
        Some(Easing::EaseIn)
    } else if keyword.eq_ignore_ascii_case("ease-out") {
        Some(Easing::EaseOut)
    } else if keyword.eq_ignore_ascii_case("ease") || keyword.eq_ignore_ascii_case("ease-in-out") {
        Some(Easing::EaseInOut)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::PropertyParser;
    use crate::source::CssSource;
    use obscur_motion::track::MotionTrack;
    use obscur_motion::value::MotionValue;

    fn compile(text: &str) -> (Vec<CompiledTransition>, Vec<UnsupportedCssFeature>) {
        let src = CssSource::anonymous(text);
        let decl = PropertyParser::new().parse(&src).unwrap();
        let mut unsupported = Vec::new();
        let transitions = TransitionCompiler::new().compile(&decl, &mut unsupported);
        (transitions, unsupported)
    }

    fn one(text: &str) -> CompiledTransition {
        let (transitions, _) = compile(text);
        assert_eq!(transitions.len(), 1, "expected exactly one transition");
        transitions[0]
    }

    #[test]
    fn parses_property_duration_and_easing() {
        let t = one("transition: opacity 0.3s ease-in");
        assert_eq!(t.property, PropertyId::Opacity);
        assert_eq!(t.duration_secs, 0.3);
        assert_eq!(t.delay_secs, 0.0);
        assert_eq!(t.easing, Easing::EaseIn);
    }

    #[test]
    fn seconds_and_milliseconds() {
        assert_eq!(one("transition: width 2s").duration_secs, 2.0);
        assert_eq!(one("transition: width 250ms").duration_secs, 0.25);
    }

    #[test]
    fn second_time_is_the_delay() {
        let t = one("transition: width 0.5s 0.1s");
        assert_eq!(t.duration_secs, 0.5);
        assert_eq!(t.delay_secs, 0.1);
    }

    #[test]
    fn zero_number_is_a_valid_time() {
        let t = one("transition: width 0 0.2s");
        assert_eq!(t.duration_secs, 0.0);
        assert_eq!(t.delay_secs, 0.2);
    }

    #[test]
    fn all_easing_keywords_map() {
        assert_eq!(one("transition: width 1s linear").easing, Easing::Linear);
        assert_eq!(one("transition: width 1s ease-in").easing, Easing::EaseIn);
        assert_eq!(one("transition: width 1s ease-out").easing, Easing::EaseOut);
        assert_eq!(one("transition: width 1s ease").easing, Easing::EaseInOut);
        assert_eq!(
            one("transition: width 1s ease-in-out").easing,
            Easing::EaseInOut
        );
    }

    #[test]
    fn missing_easing_defaults_to_linear() {
        assert_eq!(one("transition: width 1s").easing, Easing::Linear);
    }

    #[test]
    fn cubic_bezier_falls_back_and_reports_unsupported() {
        let (transitions, unsupported) =
            compile("transition: width 1s cubic-bezier(0.1, 0.2, 0.3, 0.4)");
        assert_eq!(transitions[0].easing, Easing::Linear);
        assert_eq!(unsupported.len(), 1);
        assert_eq!(unsupported[0].kind(), UnsupportedFeatureKind::Function);
        assert_eq!(unsupported[0].name(), "cubic-bezier");
    }

    #[test]
    fn steps_falls_back_and_reports_unsupported() {
        let (transitions, unsupported) = compile("transition: width 1s steps(4)");
        assert_eq!(transitions[0].easing, Easing::Linear);
        assert_eq!(unsupported.len(), 1);
        assert_eq!(unsupported[0].name(), "steps");
    }

    #[test]
    fn comma_separated_layers() {
        let (transitions, _) = compile("transition: opacity 0.3s, width 0.5s ease-out");
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].property, PropertyId::Opacity);
        assert_eq!(transitions[1].property, PropertyId::Width);
        assert_eq!(transitions[1].easing, Easing::EaseOut);
    }

    #[test]
    fn layer_without_resolvable_property_is_skipped() {
        // `all` is not one of the modelled longhands, so the layer is dropped.
        let (transitions, _) = compile("transition: all 0.3s");
        assert!(transitions.is_empty());
    }

    #[test]
    fn build_track_binds_endpoints_and_easing() {
        let template = one("transition: opacity 0.3s ease-in");
        let track = template.build_track(MotionValue::scalar(0.0), MotionValue::scalar(8.0));
        assert_eq!(track.easing(), Easing::EaseIn);
        // ease-in at t = 0.5 is 0.25, so 0..8 samples to 2.0.
        assert_eq!(track.sample(0.5), MotionValue::scalar(2.0));
        assert_eq!(track.sample(0.0), MotionValue::scalar(0.0));
        assert_eq!(track.sample(1.0), MotionValue::scalar(8.0));
    }

    #[test]
    fn unknown_property_layer_is_skipped() {
        let (transitions, _) = compile("transition: margin 0.3s");
        assert!(transitions.is_empty());
    }
}
