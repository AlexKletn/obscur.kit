#![doc = include_str!("condition_lowering.doc/_module.md")]

use crate::error::{UnsupportedCssFeature, UnsupportedFeatureKind};
use crate::parser::{
    ContainerCondition, MediaComparator, MediaCondition, MediaQualifier, ParsedContainerQuery,
    ParsedMediaQuery, ParsedMediaQueryList, ParsedValue,
};
use crate::source::SourceSpan;

use obscur_style::matching::{ConditionAxis, ConditionComparator, FeatureQuery, RuleCondition};

use super::stylesheet_compiler::CompiledCondition;

#[doc = include_str!("condition_lowering.doc/lower_condition.md")]
pub(crate) fn lower_condition(
    condition: &CompiledCondition,
    unsupported: &mut Vec<UnsupportedCssFeature>,
) -> RuleCondition {
    match condition {
        CompiledCondition::Media(list) => lower_media_list(list, unsupported),
        CompiledCondition::Container(query) => lower_container_query(query),
    }
}

#[doc = include_str!("condition_lowering.doc/lower_media_list.md")]
fn lower_media_list(
    list: &ParsedMediaQueryList,
    unsupported: &mut Vec<UnsupportedCssFeature>,
) -> RuleCondition {
    match list.0.as_slice() {
        [] => RuleCondition::Always,
        [single] => lower_media_query(single, unsupported),
        many => RuleCondition::Any(
            many.iter()
                .map(|query| lower_media_query(query, unsupported))
                .collect(),
        ),
    }
}

#[doc = include_str!("condition_lowering.doc/lower_media_query.md")]
fn lower_media_query(
    query: &ParsedMediaQuery,
    unsupported: &mut Vec<UnsupportedCssFeature>,
) -> RuleCondition {
    if query
        .media_type
        .as_deref()
        .is_some_and(|ty| ty.eq_ignore_ascii_case("print"))
    {
        // `print` rules never apply in the (screen) runtime environment, and
        // that is knowledge, not ignorance: `not print` may negate it.
        return qualify(query.qualifier, RuleCondition::Never);
    }
    let mut parts = Vec::with_capacity(query.conditions.len());
    let mut representable = true;
    for condition in &query.conditions {
        let Some(part) = lower_media_condition(condition) else {
            representable = false;
            unsupported.push(unrepresentable_media_feature(&condition.feature));
            continue;
        };
        parts.push(part);
    }
    if !representable {
        // Negating an unanswerable question turns ignorance into a claim, so
        // the `not` qualifier is deliberately not applied here.
        return RuleCondition::Never;
    }
    qualify(query.qualifier, conjoin(parts.into_iter()))
}

#[doc = include_str!("condition_lowering.doc/qualify.md")]
fn qualify(qualifier: Option<MediaQualifier>, base: RuleCondition) -> RuleCondition {
    match qualifier {
        Some(MediaQualifier::Not) => RuleCondition::Not(Box::new(base)),
        // `only` is a legacy no-op; absence means the query applies as-is.
        Some(MediaQualifier::Only) | None => base,
    }
}

#[doc = include_str!("condition_lowering.doc/unrepresentable_media_feature.md")]
fn unrepresentable_media_feature(feature: &str) -> UnsupportedCssFeature {
    // The parsed condition carries no offsets, so the report can only name the
    // feature; an empty span at the sheet start is the honest placeholder.
    UnsupportedCssFeature::new(
        UnsupportedFeatureKind::AtRule,
        format!("@media ({feature})"),
        SourceSpan::empty(0),
    )
}

#[doc = include_str!("condition_lowering.doc/lower_container_query.md")]
fn lower_container_query(query: &ParsedContainerQuery) -> RuleCondition {
    if query.conditions.is_empty() {
        return RuleCondition::Never;
    }
    RuleCondition::All(
        query
            .conditions
            .iter()
            .map(lower_container_condition)
            .collect(),
    )
}

#[doc = include_str!("condition_lowering.doc/conjoin.md")]
fn conjoin(parts: impl Iterator<Item = RuleCondition>) -> RuleCondition {
    let parts: Vec<RuleCondition> = parts.collect();
    if parts.is_empty() {
        RuleCondition::Always
    } else {
        RuleCondition::All(parts)
    }
}

#[doc = include_str!("condition_lowering.doc/lower_media_condition.md")]
fn lower_media_condition(condition: &MediaCondition) -> Option<RuleCondition> {
    lower_feature(
        &condition.feature,
        condition.comparator,
        condition.value.as_deref(),
        ConditionAxis::ViewportWidth,
        ConditionAxis::ViewportHeight,
    )
}

#[doc = include_str!("condition_lowering.doc/lower_container_condition.md")]
fn lower_container_condition(condition: &ContainerCondition) -> RuleCondition {
    lower_feature(
        &condition.feature,
        condition.comparator,
        condition.value.as_deref(),
        ConditionAxis::ContainerWidth,
        ConditionAxis::ContainerHeight,
    )
    .unwrap_or(RuleCondition::Never)
}

#[doc = include_str!("condition_lowering.doc/lower_feature.md")]
fn lower_feature(
    feature: &str,
    comparator: Option<MediaComparator>,
    value: Option<&[ParsedValue]>,
    width_axis: ConditionAxis,
    height_axis: ConditionAxis,
) -> Option<RuleCondition> {
    let name = feature.to_ascii_lowercase();
    let (axis, synthesized) = match name.as_str() {
        "min-width" | "min-inline-size" => (width_axis, Some(ConditionComparator::Ge)),
        "max-width" | "max-inline-size" => (width_axis, Some(ConditionComparator::Le)),
        "width" | "inline-size" => (width_axis, None),
        "min-height" | "min-block-size" => (height_axis, Some(ConditionComparator::Ge)),
        "max-height" | "max-block-size" => (height_axis, Some(ConditionComparator::Le)),
        "height" | "block-size" => (height_axis, None),
        // Boolean features, orientation, prefers-*, unknown names.
        _ => return None,
    };
    let reference_px = value.and_then(reference_px)?;
    // `min-`/`max-` forms carry their own comparator; plain `width`/`height` use
    // the range comparator, defaulting a bare `(width: 600px)` colon form to `=`.
    let comparator = synthesized
        .or_else(|| comparator.map(map_comparator))
        .unwrap_or(ConditionComparator::Eq);
    Some(RuleCondition::Feature(FeatureQuery::new(
        axis,
        comparator,
        reference_px,
    )))
}

#[doc = include_str!("condition_lowering.doc/reference_px.md")]
fn reference_px(value: &[ParsedValue]) -> Option<f32> {
    match value.first()? {
        ParsedValue::Dimension { value, unit } if unit.eq_ignore_ascii_case("px") => {
            Some(*value as f32)
        }
        // CSS permits a unitless zero length.
        ParsedValue::Number(number) if *number == 0.0 => Some(0.0),
        _ => None,
    }
}

fn map_comparator(comparator: MediaComparator) -> ConditionComparator {
    match comparator {
        MediaComparator::Eq => ConditionComparator::Eq,
        MediaComparator::Lt => ConditionComparator::Lt,
        MediaComparator::Le => ConditionComparator::Le,
        MediaComparator::Gt => ConditionComparator::Gt,
        MediaComparator::Ge => ConditionComparator::Ge,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ContainerQueryParser, MediaQueryParser};
    use crate::source::CssSource;
    use obscur_style::matching::MediaEnvironment;

    fn media_condition(text: &str) -> RuleCondition {
        lower_media(text).0
    }

    fn lower_media(text: &str) -> (RuleCondition, Vec<UnsupportedCssFeature>) {
        let src = CssSource::anonymous(text);
        let list = MediaQueryParser::new().parse(&src).unwrap();
        let mut unsupported = Vec::new();
        let cond = lower_condition(&CompiledCondition::Media(list), &mut unsupported);
        (cond, unsupported)
    }

    fn container_condition(text: &str) -> RuleCondition {
        let src = CssSource::anonymous(text);
        let query = ContainerQueryParser::new().parse(&src).unwrap();
        lower_condition(&CompiledCondition::Container(query), &mut Vec::new())
    }

    #[test]
    fn max_width_gates_by_viewport() {
        let cond = media_condition("(max-width: 1px)");
        assert!(cond.holds(&MediaEnvironment::viewport(1.0, 1000.0)));
        assert!(!cond.holds(&MediaEnvironment::viewport(1000.0, 1000.0)));
    }

    #[test]
    fn min_width_gates_by_viewport() {
        let cond = media_condition("screen and (min-width: 600px)");
        assert!(cond.holds(&MediaEnvironment::viewport(800.0, 600.0)));
        assert!(!cond.holds(&MediaEnvironment::viewport(400.0, 600.0)));
    }

    #[test]
    fn range_feature_maps_comparator() {
        let cond = media_condition("(width < 400px)");
        assert!(cond.holds(&MediaEnvironment::viewport(399.0, 100.0)));
        assert!(!cond.holds(&MediaEnvironment::viewport(400.0, 100.0)));
    }

    #[test]
    fn two_conditions_are_conjunctive() {
        let cond = media_condition("(min-width: 400px) and (max-width: 800px)");
        assert!(cond.holds(&MediaEnvironment::viewport(600.0, 100.0)));
        assert!(!cond.holds(&MediaEnvironment::viewport(900.0, 100.0)));
        assert!(!cond.holds(&MediaEnvironment::viewport(300.0, 100.0)));
    }

    #[test]
    fn query_list_is_disjunctive() {
        let cond = media_condition("(max-width: 100px), (min-width: 900px)");
        assert!(cond.holds(&MediaEnvironment::viewport(50.0, 100.0)));
        assert!(cond.holds(&MediaEnvironment::viewport(1000.0, 100.0)));
        assert!(!cond.holds(&MediaEnvironment::viewport(500.0, 100.0)));
    }

    #[test]
    fn print_never_applies() {
        let cond = media_condition("print");
        assert!(!cond.holds(&MediaEnvironment::viewport(500.0, 500.0)));
    }

    #[test]
    fn not_print_always_applies_on_screen() {
        let cond = media_condition("not print");
        assert!(cond.holds(&MediaEnvironment::viewport(500.0, 500.0)));
    }

    #[test]
    fn unrepresentable_media_feature_fails_closed() {
        // A feature we cannot evaluate must not be answered "yes". Applying a
        // dark theme or a phone layout unconditionally is the loudest possible
        // way to be wrong, and it is silent to the author.
        let env = MediaEnvironment::viewport(10000.0, 500.0);
        assert!(!media_condition("(prefers-color-scheme: dark)").holds(&env));
        assert!(!media_condition("(prefers-reduced-motion: reduce)").holds(&env));
        assert!(!media_condition("(orientation: landscape)").holds(&env));
        assert!(!media_condition("(hover)").holds(&env));
        assert!(!media_condition("(hover)").holds(&MediaEnvironment::unbounded()));
    }

    #[test]
    fn non_px_length_fails_closed() {
        // `reference_px` resolves only `px` (and a unitless zero), so `40em` is
        // as unevaluable as a boolean feature.
        let cond = media_condition("(max-width: 40em)");
        assert!(!cond.holds(&MediaEnvironment::viewport(10000.0, 500.0)));
        assert!(!cond.holds(&MediaEnvironment::viewport(100.0, 500.0)));
    }

    #[test]
    fn an_unrepresentable_feature_is_reported_as_unsupported() {
        let (_, unsupported) = lower_media("(prefers-color-scheme: dark)");
        assert_eq!(unsupported.len(), 1);
        assert_eq!(unsupported[0].kind(), UnsupportedFeatureKind::AtRule);
        assert_eq!(unsupported[0].name(), "@media (prefers-color-scheme)");
    }

    #[test]
    fn every_unrepresentable_feature_in_a_query_is_reported() {
        let (cond, unsupported) = lower_media("(orientation: landscape) and (hover)");
        assert_eq!(cond, RuleCondition::Never);
        let names: Vec<&str> = unsupported
            .iter()
            .map(UnsupportedCssFeature::name)
            .collect();
        assert_eq!(names, ["@media (orientation)", "@media (hover)"]);
    }

    #[test]
    fn a_representable_query_reports_nothing() {
        let (_, unsupported) = lower_media("screen and (min-width: 600px)");
        assert!(unsupported.is_empty());
        assert!(lower_media("print").1.is_empty());
    }

    #[test]
    fn not_does_not_rescue_an_unrepresentable_feature() {
        // `Not(Never)` is `Always`: negating what we could not evaluate would
        // reintroduce the always-applies bug in the opposite polarity.
        let (cond, unsupported) = lower_media("not all and (prefers-color-scheme: dark)");
        assert_eq!(cond, RuleCondition::Never);
        assert!(!cond.holds(&MediaEnvironment::viewport(500.0, 500.0)));
        assert_eq!(unsupported.len(), 1);
    }

    #[test]
    fn one_unrepresentable_query_does_not_sink_the_list() {
        // A list is a disjunction: the queries we can evaluate still apply.
        let cond = media_condition("(max-width: 100px), (orientation: portrait)");
        assert!(cond.holds(&MediaEnvironment::viewport(50.0, 100.0)));
        assert!(!cond.holds(&MediaEnvironment::viewport(500.0, 100.0)));
    }

    #[test]
    fn container_query_fails_closed_without_a_container() {
        // The live runtime supplies a viewport but never a container size, so
        // this is the environment every `@container` rule is really evaluated
        // against. It must not hold.
        let env = MediaEnvironment::viewport(1000.0, 1000.0);
        assert!(!container_condition("(min-inline-size: 400px)").holds(&env));
        assert!(!container_condition("(max-width: 400px)").holds(&env));
        assert!(!container_condition("(width > 100px)").holds(&env));
    }

    #[test]
    fn unrepresentable_container_feature_fails_closed() {
        // The back door: a feature we cannot lower to a length must not fall
        // back to `Always` inside a container query.
        let env = MediaEnvironment::viewport(1000.0, 1000.0);
        assert!(!container_condition("(orientation: portrait)").holds(&env));
        assert!(!container_condition("(max-inline-size: 40em)").holds(&env));
        // Even with a container supplied, an untestable feature cannot be said
        // to hold.
        assert!(
            !container_condition("(orientation: portrait)")
                .holds(&env.with_container(500.0, 500.0))
        );
    }

    #[test]
    fn bare_named_container_query_fails_closed() {
        // `@container sidebar { … }` asks only whether the container exists.
        let cond = container_condition("sidebar");
        assert_eq!(cond, RuleCondition::Never);
        assert!(!cond.holds(&MediaEnvironment::viewport(1000.0, 1000.0)));
    }

    #[test]
    fn container_inline_size_gates() {
        let cond = container_condition("(min-inline-size: 400px)");
        let env = MediaEnvironment::viewport(1000.0, 1000.0);
        assert!(!cond.holds(&env.with_container(200.0, 200.0)));
        assert!(cond.holds(&env.with_container(500.0, 200.0)));
        // Unknown container size (viewport-only env) fails closed.
        assert!(!cond.holds(&env));
    }
}
