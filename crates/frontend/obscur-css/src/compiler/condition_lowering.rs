//! Lowering a parsed `@media`/`@container` condition into `obscur-style`'s
//! CSS-syntax-free [`RuleCondition`], so the runtime matcher can gate rules by a
//! live environment (§4.13).
//!
//! Only length features on the width/height (viewport) or inline-size/block-size
//! (container) axes with a `px` reference are represented as comparisons. Every
//! other feature (a boolean feature, a non-`px` length, an unknown name) lowers
//! to a per-family fallback: [`RuleCondition::Always`] for `@media`, so the rule
//! is gated permissively rather than silently dropped, but
//! [`RuleCondition::Never`] for `@container`, which fails closed because nothing
//! can establish a query container — see
//! [`MediaEnvironment`](obscur_style::matching::MediaEnvironment).
//!
//! `and`/`or` connectors are not distinguished by the parser, so a query's
//! feature list is treated as a conjunction; a media query list is a
//! disjunction, and the `not` qualifier negates a query. Container queries have
//! no `not` qualifier in the parsed grammar, so no negation can flip a
//! failed-closed container feature back to holding.

use crate::parser::{
    ContainerCondition, MediaComparator, MediaCondition, MediaQualifier, ParsedContainerQuery,
    ParsedMediaQuery, ParsedMediaQueryList, ParsedValue,
};

use obscur_style::matching::{ConditionAxis, ConditionComparator, FeatureQuery, RuleCondition};

use super::stylesheet_compiler::CompiledCondition;

/// Lowers a compiled condition into an evaluable [`RuleCondition`].
pub(crate) fn lower_condition(condition: &CompiledCondition) -> RuleCondition {
    match condition {
        CompiledCondition::Media(list) => lower_media_list(list),
        CompiledCondition::Container(query) => lower_container_query(query),
    }
}

/// A media query list is a disjunction of its queries.
fn lower_media_list(list: &ParsedMediaQueryList) -> RuleCondition {
    match list.0.as_slice() {
        [] => RuleCondition::Always,
        [single] => lower_media_query(single),
        many => RuleCondition::Any(many.iter().map(lower_media_query).collect()),
    }
}

/// Lowers one media query: its media type plus its (conjunctive) features,
/// negated when the query carries the `not` qualifier.
fn lower_media_query(query: &ParsedMediaQuery) -> RuleCondition {
    let base = if query
        .media_type
        .as_deref()
        .is_some_and(|ty| ty.eq_ignore_ascii_case("print"))
    {
        // `print` rules never apply in the (screen) runtime environment.
        RuleCondition::Never
    } else {
        conjoin(
            query
                .conditions
                .iter()
                .map(|cond| lower_media_condition(cond)),
        )
    };
    match query.qualifier {
        Some(MediaQualifier::Not) => RuleCondition::Not(Box::new(base)),
        // `only` is a legacy no-op; absence means the query applies as-is.
        Some(MediaQualifier::Only) | None => base,
    }
}

/// A container query is a conjunction of its features.
///
/// The container name is ignored: nothing can establish a *named* container any
/// more than an anonymous one, so matching the name would not change the answer.
/// A query with no features at all (`@container name { … }`) asks only "does
/// this container exist", which is [`Never`](RuleCondition::Never) here.
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

/// Builds a conjunction, collapsing the empty case to [`RuleCondition::Always`].
fn conjoin(parts: impl Iterator<Item = RuleCondition>) -> RuleCondition {
    let parts: Vec<RuleCondition> = parts.collect();
    if parts.is_empty() {
        RuleCondition::Always
    } else {
        RuleCondition::All(parts)
    }
}

/// Lowers a media feature (viewport width/height axes).
///
/// An unrepresentable media feature is permissive: the viewport exists, we
/// simply cannot test this aspect of it, so the rule still applies.
fn lower_media_condition(condition: &MediaCondition) -> RuleCondition {
    lower_feature(
        &condition.feature,
        condition.comparator,
        condition.value.as_deref(),
        ConditionAxis::ViewportWidth,
        ConditionAxis::ViewportHeight,
        RuleCondition::Always,
    )
}

/// Lowers a container feature (container inline/block size axes).
///
/// An unrepresentable container feature lowers to [`RuleCondition::Never`], not
/// `Always`: a container query is evaluated against a container that nothing in
/// this pipeline can establish, so a permissive fallback would resurrect the
/// always-matches bug through the back door (`@container (orientation: portrait)`
/// would apply everywhere).
fn lower_container_condition(condition: &ContainerCondition) -> RuleCondition {
    lower_feature(
        &condition.feature,
        condition.comparator,
        condition.value.as_deref(),
        ConditionAxis::ContainerWidth,
        ConditionAxis::ContainerHeight,
        RuleCondition::Never,
    )
}

/// Maps a feature name/comparator/value onto a width- or height-axis comparison,
/// falling back to `unrepresentable` for anything it cannot express.
fn lower_feature(
    feature: &str,
    comparator: Option<MediaComparator>,
    value: Option<&[ParsedValue]>,
    width_axis: ConditionAxis,
    height_axis: ConditionAxis,
    unrepresentable: RuleCondition,
) -> RuleCondition {
    let name = feature.to_ascii_lowercase();
    let (axis, synthesized) = match name.as_str() {
        "min-width" | "min-inline-size" => (width_axis, Some(ConditionComparator::Ge)),
        "max-width" | "max-inline-size" => (width_axis, Some(ConditionComparator::Le)),
        "width" | "inline-size" => (width_axis, None),
        "min-height" | "min-block-size" => (height_axis, Some(ConditionComparator::Ge)),
        "max-height" | "max-block-size" => (height_axis, Some(ConditionComparator::Le)),
        "height" | "block-size" => (height_axis, None),
        // Boolean features, orientation, prefers-*, unknown names.
        _ => return unrepresentable,
    };
    let Some(reference_px) = value.and_then(reference_px) else {
        return unrepresentable;
    };
    // `min-`/`max-` forms carry their own comparator; plain `width`/`height` use
    // the range comparator, defaulting a bare `(width: 600px)` colon form to `=`.
    let comparator = synthesized
        .or_else(|| comparator.map(map_comparator))
        .unwrap_or(ConditionComparator::Eq);
    RuleCondition::Feature(FeatureQuery::new(axis, comparator, reference_px))
}

/// Extracts a single `px` reference length (or unitless `0`) from a feature
/// value, returning [`None`] for anything we cannot resolve to pixels.
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
        let src = CssSource::anonymous(text);
        let list = MediaQueryParser::new().parse(&src).unwrap();
        lower_condition(&CompiledCondition::Media(list))
    }

    fn container_condition(text: &str) -> RuleCondition {
        let src = CssSource::anonymous(text);
        let query = ContainerQueryParser::new().parse(&src).unwrap();
        lower_condition(&CompiledCondition::Container(query))
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
    fn boolean_feature_is_non_restrictive() {
        // `(hover)` cannot be resolved to a length; it must not drop the rule.
        let cond = media_condition("(hover)");
        assert!(cond.holds(&MediaEnvironment::viewport(500.0, 500.0)));
        assert!(cond.holds(&MediaEnvironment::unbounded()));
    }

    #[test]
    fn non_px_length_is_non_restrictive() {
        let cond = media_condition("(max-width: 40em)");
        assert!(cond.holds(&MediaEnvironment::viewport(10000.0, 500.0)));
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
