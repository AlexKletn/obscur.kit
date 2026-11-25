#![doc = include_str!("../../docs/api/compiler/condition_lowering.doc/_module.md")]

use crate::error::{UnsupportedCssFeature, UnsupportedFeatureKind};
use crate::parser::{
    ContainerCondition, ContainerGroup, ContainerLogic, MediaComparator, MediaCondition,
    MediaLogic, MediaQualifier, ParsedContainerQuery, ParsedMediaQuery, ParsedMediaQueryList,
    ParsedValue,
};
use crate::source::SourceSpan;
use obscur_identity::InternedSymbol;
use obscur_style::property::{PropertyId, PropertyValue};

use super::CssSymbolInterner;
use super::longhand_catalog::property_id_from_name;
use super::property_compiler::lower_for_property;

use super::custom_value::{custom_value_text, lower_style_range};
use super::value_lowering::sole_custom_ref;

use obscur_style::matching::{
    ConditionAxis, ConditionComparator, FeatureQuery, RuleCondition, StylePropertyValue,
    StyleQueryValue, StyleRangeBound,
};

use super::CompiledCondition;

/// Lowers a compiled condition into an evaluable [`RuleCondition`], recording any
/// media feature it could not represent into `unsupported`.
pub(crate) fn lower_condition(
    condition: &CompiledCondition,
    unsupported: &mut Vec<UnsupportedCssFeature>,
) -> RuleCondition {
    match condition {
        CompiledCondition::Media(list) => lower_media_list(list, unsupported),
        CompiledCondition::Container(query) => lower_container_query(query),
    }
}

#[doc = include_str!("../../docs/api/compiler/condition_lowering.doc/lower_media_list.md")]
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

#[doc = include_str!("../../docs/api/compiler/condition_lowering.doc/lower_media_query.md")]
fn lower_media_query(
    query: &ParsedMediaQuery,
    unsupported: &mut Vec<UnsupportedCssFeature>,
) -> RuleCondition {
    if query.media_type.as_deref().is_some_and(|kind| {
        !kind.eq_ignore_ascii_case("all") && !kind.eq_ignore_ascii_case("screen")
    }) {
        return qualify(query.qualifier, RuleCondition::Never);
    }
    let parts: Vec<_> = query
        .conditions
        .iter()
        .map(|condition| {
            lower_media_condition(condition).unwrap_or_else(|| {
                unsupported.push(unrepresentable_media_feature(&condition.feature));
                RuleCondition::Unknown(None)
            })
        })
        .collect();
    let condition = if let Some(logic) = &query.logic {
        lower_media_logic(logic, &parts)
    } else {
        conjoin(parts.into_iter())
    };
    qualify(query.qualifier, condition)
}

/// Lowers nested media operators while preserving unknown feature values.
fn lower_media_logic(logic: &MediaLogic, parts: &[RuleCondition]) -> RuleCondition {
    match logic {
        MediaLogic::Leaf(index) => parts
            .get(*index)
            .cloned()
            .unwrap_or(RuleCondition::Unknown(None)),
        MediaLogic::All(children) => {
            conjoin(children.iter().map(|child| lower_media_logic(child, parts)))
        }
        MediaLogic::Any(children) => RuleCondition::Any(
            children
                .iter()
                .map(|child| lower_media_logic(child, parts))
                .collect(),
        ),
        MediaLogic::Not(child) => RuleCondition::Not(Box::new(lower_media_logic(child, parts))),
    }
}

/// Applies a query's leading qualifier to an already-lowered condition.
fn qualify(qualifier: Option<MediaQualifier>, base: RuleCondition) -> RuleCondition {
    match qualifier {
        Some(MediaQualifier::Not) => RuleCondition::Not(Box::new(base)),
        // `only` is a legacy no-op; absence means the query applies as-is.
        Some(MediaQualifier::Only) | None => base,
    }
}

#[doc = include_str!("../../docs/api/compiler/condition_lowering.doc/unrepresentable_media_feature.md")]
fn unrepresentable_media_feature(feature: &str) -> UnsupportedCssFeature {
    // The parsed condition carries no offsets, so the report can only name the
    // feature; an empty span at the sheet start is the honest placeholder.
    UnsupportedCssFeature::new(
        UnsupportedFeatureKind::AtRule,
        format!("@media ({feature})"),
        SourceSpan::empty(0),
    )
}

/// Preserves the optional container name and the logical structure of its size
/// and style conditions. An empty query lowers to [`Never`](RuleCondition::Never).
fn lower_container_query(query: &ParsedContainerQuery) -> RuleCondition {
    if query.conditions.is_empty() {
        return RuleCondition::Never;
    }
    let parts: Vec<_> = query
        .conditions
        .iter()
        .map(lower_container_condition)
        .collect();
    let condition = match &query.logic {
        ContainerLogic::All => RuleCondition::All(parts),
        ContainerLogic::Any => RuleCondition::Any(parts),
        ContainerLogic::Not => match parts.as_slice() {
            [condition] => RuleCondition::Not(Box::new(condition.clone())),
            _ => return RuleCondition::Never,
        },
        ContainerLogic::Nested(group) => lower_container_group(group, &parts),
    };
    RuleCondition::Container {
        name: query.name.clone(),
        condition: Box::new(condition),
    }
}

fn lower_container_group(group: &ContainerGroup, parts: &[RuleCondition]) -> RuleCondition {
    match group {
        ContainerGroup::Leaf(index) => parts
            .get(*index)
            .cloned()
            .unwrap_or(RuleCondition::Unknown(None)),
        ContainerGroup::Range([first, second]) => match (parts.get(*first), parts.get(*second)) {
            (
                Some(RuleCondition::ContainerStyleRange {
                    property,
                    comparator: left,
                    reference: a,
                }),
                Some(RuleCondition::ContainerStyleRange {
                    property: other,
                    comparator: right,
                    reference: b,
                }),
            ) if property == other => RuleCondition::ContainerStyleChain {
                property: *property,
                bounds: [(*left, a.clone()), (*right, b.clone())],
            },
            _ => RuleCondition::Unknown(None),
        },
        ContainerGroup::All(children) => RuleCondition::All(
            children
                .iter()
                .map(|child| lower_container_group(child, parts))
                .collect(),
        ),
        ContainerGroup::Any(children) => RuleCondition::Any(
            children
                .iter()
                .map(|child| lower_container_group(child, parts))
                .collect(),
        ),
        ContainerGroup::Not(child) => {
            RuleCondition::Not(Box::new(lower_container_group(child, parts)))
        }
    }
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

#[doc = include_str!("../../docs/api/compiler/condition_lowering.doc/lower_media_condition.md")]
fn lower_media_condition(condition: &MediaCondition) -> Option<RuleCondition> {
    let feature = condition.feature.to_ascii_lowercase();
    if matches!(
        feature.as_str(),
        "resolution" | "min-resolution" | "max-resolution"
    ) {
        return lower_resolution(&feature, condition);
    }
    if matches!(
        feature.as_str(),
        "pointer" | "any-pointer" | "hover" | "any-hover"
    ) {
        return lower_media_input(&feature, condition);
    }
    if condition.feature.eq_ignore_ascii_case("orientation") {
        if condition.comparator.is_some() {
            return None;
        }
        return match condition.value.as_deref() {
            Some([ParsedValue::Keyword(value)]) if value.eq_ignore_ascii_case("landscape") => {
                Some(RuleCondition::Landscape(true))
            }
            Some([ParsedValue::Keyword(value)]) if value.eq_ignore_ascii_case("portrait") => {
                Some(RuleCondition::Landscape(false))
            }
            None => Some(RuleCondition::Any(vec![
                RuleCondition::Landscape(true),
                RuleCondition::Landscape(false),
            ])),
            _ => None,
        };
    }
    let (check, active, inactive): (fn(bool) -> RuleCondition, &str, &str) = match feature.as_str()
    {
        "prefers-color-scheme" => (RuleCondition::PrefersDark, "dark", "light"),
        "prefers-reduced-motion" => (
            RuleCondition::PrefersReducedMotion,
            "reduce",
            "no-preference",
        ),
        "prefers-contrast" => (
            RuleCondition::PrefersIncreasedContrast,
            "more",
            "no-preference",
        ),
        _ => {
            return lower_feature(
                &condition.feature,
                condition.comparator,
                condition.value.as_deref(),
                ConditionAxis::ViewportWidth,
                ConditionAxis::ViewportHeight,
            );
        }
    };
    if condition.comparator.is_some() {
        return None;
    }
    match condition.value.as_deref() {
        Some([ParsedValue::Keyword(value)]) if value.eq_ignore_ascii_case(active) => {
            Some(check(true))
        }
        Some([ParsedValue::Keyword(value)]) if value.eq_ignore_ascii_case(inactive) => {
            Some(check(false))
        }
        Some([ParsedValue::Keyword(value)])
            if feature == "prefers-contrast"
                && (value.eq_ignore_ascii_case("less") || value.eq_ignore_ascii_case("custom")) =>
        {
            Some(RuleCondition::Never)
        }
        None if feature == "prefers-color-scheme" => Some(RuleCondition::Always),
        None => Some(check(true)),
        _ => None,
    }
}

/// Lowers discrete pointer and hover queries for the primary or any pointing device.
fn lower_media_input(feature: &str, condition: &MediaCondition) -> Option<RuleCondition> {
    use obscur_style::matching::PointerAccuracy;

    if condition.comparator.is_some() {
        return None;
    }
    let keyword = match condition.value.as_deref() {
        Some([ParsedValue::Keyword(value)]) => Some(value.to_ascii_lowercase()),
        None => None,
        _ => return None,
    };
    let any = feature.starts_with("any-");
    if matches!(feature, "hover" | "any-hover") {
        return match keyword.as_deref() {
            None | Some("hover") => Some(RuleCondition::Hover { any, enabled: true }),
            Some("none") => Some(RuleCondition::Hover {
                any,
                enabled: false,
            }),
            _ => None,
        };
    }
    let (accuracy, present) = match keyword.as_deref() {
        None => (None, true),
        Some("none") => (None, false),
        Some("coarse") => (Some(PointerAccuracy::Coarse), true),
        Some("fine") => (Some(PointerAccuracy::Fine), true),
        _ => return None,
    };
    Some(RuleCondition::Pointer {
        any,
        accuracy,
        present,
    })
}

/// Lowers resolution bounds to the canonical screen pixel ratio in dppx.
fn lower_resolution(feature: &str, condition: &MediaCondition) -> Option<RuleCondition> {
    let comparator = match feature {
        "min-resolution" if condition.comparator.is_none() => ConditionComparator::Ge,
        "max-resolution" if condition.comparator.is_none() => ConditionComparator::Le,
        "resolution" => condition
            .comparator
            .map_or(ConditionComparator::Eq, map_comparator),
        _ => return None,
    };
    let dppx = match condition.value.as_deref() {
        None if feature == "resolution" && condition.comparator.is_none() => {
            return Some(RuleCondition::Resolution(ConditionComparator::Gt, 0.0));
        }
        Some([ParsedValue::Keyword(value)]) if value.eq_ignore_ascii_case("infinite") => {
            f32::INFINITY
        }
        Some([ParsedValue::Dimension { value, unit }]) if value.is_finite() && *value >= 0.0 => {
            let value = match unit.to_ascii_lowercase().as_str() {
                "dppx" | "x" => *value,
                "dpi" => *value / 96.0,
                "dpcm" => *value * (2.54 / 96.0),
                _ => return None,
            };
            super::property_compiler::narrow_f32(value)
        }
        _ => return None,
    };
    Some(RuleCondition::Resolution(comparator, dppx))
}

/// Lowers a single size feature and makes unsupported features fail closed.
fn lower_container_condition(condition: &ContainerCondition) -> RuleCondition {
    if condition.style {
        if !condition.feature.starts_with("--") {
            return lower_standard_style_test(condition).unwrap_or(RuleCondition::Unknown(None));
        }
        if let Some(comparator) = condition.comparator {
            return condition
                .value
                .as_deref()
                .and_then(lower_style_bound)
                .map_or(RuleCondition::Unknown(None), |reference| {
                    RuleCondition::ContainerStyleRange {
                        property: InternedSymbol::named(&condition.feature),
                        comparator: map_comparator(comparator),
                        reference,
                    }
                });
        }
        return RuleCondition::ContainerStyle {
            property: InternedSymbol::named(&condition.feature),
            value: condition.value.as_deref().map(|tokens| {
                sole_custom_ref(tokens)
                    .unwrap_or_else(|| PropertyValue::RawTokens(custom_value_text(tokens)))
            }),
        };
    }
    lower_feature(
        &condition.feature,
        condition.comparator,
        condition.value.as_deref(),
        ConditionAxis::ContainerWidth,
        ConditionAxis::ContainerHeight,
    )
    .unwrap_or_else(|| {
        let axis = feature_axis(
            &condition.feature,
            ConditionAxis::ContainerWidth,
            ConditionAxis::ContainerHeight,
        )
        .map(|(axis, _)| axis);
        RuleCondition::Unknown(axis)
    })
}

fn lower_style_bound(values: &[ParsedValue]) -> Option<StyleRangeBound> {
    lower_style_operand(values, lower_style_range)
}

/// Identifies properties whose styles and queries share reserved keyword symbols.
fn is_style_query_keyword(property: PropertyId) -> bool {
    matches!(
        property,
        PropertyId::Display
            | PropertyId::Position
            | PropertyId::BoxSizing
            | PropertyId::Direction
            | PropertyId::WritingMode
            | PropertyId::Visibility
            | PropertyId::PointerEvents
            | PropertyId::Isolation
            | PropertyId::MixBlendMode
            | PropertyId::ZIndex
            | PropertyId::FlexDirection
            | PropertyId::FlexWrap
            | PropertyId::JustifyContent
            | PropertyId::AlignContent
            | PropertyId::AlignItems
            | PropertyId::JustifyItems
            | PropertyId::AlignSelf
            | PropertyId::JustifySelf
            | PropertyId::OverflowX
            | PropertyId::OverflowY
            | PropertyId::OverscrollBehaviorX
            | PropertyId::OverscrollBehaviorY
            | PropertyId::TextAlign
            | PropertyId::TextOverflow
            | PropertyId::TextTransform
            | PropertyId::WhiteSpace
            | PropertyId::OverflowWrap
            | PropertyId::WordBreak
    )
}

/// Identifies the standard style-query properties that accept a number.
fn is_style_query_number(property: PropertyId) -> bool {
    matches!(
        property,
        PropertyId::Opacity
            | PropertyId::Order
            | PropertyId::ZIndex
            | PropertyId::FlexGrow
            | PropertyId::FlexShrink
    )
}

fn lower_standard_style_test(condition: &ContainerCondition) -> Option<RuleCondition> {
    let property = property_id_from_name(&condition.feature)?;
    if !is_style_query_keyword(property)
        && !property.supports_style_query_length()
        && !is_style_query_number(property)
        && !matches!(property, PropertyId::Color | PropertyId::BackgroundColor)
    {
        return None;
    }
    let value = match &condition.value {
        None => None,
        Some(tokens) => Some(lower_expected_style_value(property, tokens)?),
    };
    Some(RuleCondition::ContainerProperty { property, value })
}

fn lower_expected_style_value(
    property: PropertyId,
    tokens: &[ParsedValue],
) -> Option<StylePropertyValue> {
    lower_style_operand(tokens, |tokens| lower_style_literal(property, tokens))
}

fn lower_style_operand<T>(
    mut tokens: &[ParsedValue],
    lower_literal: impl Fn(&[ParsedValue]) -> Option<T>,
) -> Option<StyleQueryValue<T>> {
    let mut references = Vec::new();
    let mut value = loop {
        let [ParsedValue::Function { name, args }] = tokens else {
            break Some(StyleQueryValue::Literal(lower_literal(tokens)?));
        };
        if !name.eq_ignore_ascii_case("var") {
            break Some(StyleQueryValue::Literal(lower_literal(tokens)?));
        }
        let [ParsedValue::Keyword(name), rest @ ..] = args.as_slice() else {
            return None;
        };
        if !name.starts_with("--") || name.len() == 2 {
            return None;
        }
        references.push(InternedSymbol::named(name));
        match rest {
            [] => break None,
            [ParsedValue::Comma, fallback @ ..] => tokens = fallback,
            _ => return None,
        }
    };
    while let Some(name) = references.pop() {
        value = Some(StyleQueryValue::Variable {
            name,
            fallback: value.map(Box::new),
        });
    }
    value
}

/// Checks whether a reserved sizing keyword can be compared for this property.
fn is_style_query_sizing_keyword(property: PropertyId, symbol: InternedSymbol) -> bool {
    use obscur_style::property::layout_keyword::{
        SIZING_FIT_CONTENT, SIZING_MAX_CONTENT, SIZING_MIN_CONTENT, SIZING_NONE,
    };

    match symbol {
        SIZING_NONE => matches!(property, PropertyId::MaxWidth | PropertyId::MaxHeight),
        SIZING_MIN_CONTENT | SIZING_MAX_CONTENT | SIZING_FIT_CONTENT => matches!(
            property,
            PropertyId::Width
                | PropertyId::Height
                | PropertyId::MinWidth
                | PropertyId::MinHeight
                | PropertyId::MaxWidth
                | PropertyId::MaxHeight
                | PropertyId::FlexBasis
        ),
        _ => false,
    }
}

/// Validates and lowers a standard style-query comparison value.
pub(super) fn lower_style_literal(
    property: PropertyId,
    tokens: &[ParsedValue],
) -> Option<PropertyValue> {
    if tokens.len() != 1
        && !(matches!(
            property,
            PropertyId::AlignItems
                | PropertyId::JustifyItems
                | PropertyId::AlignSelf
                | PropertyId::JustifySelf
        ) && super::property_compiler::baseline_position(tokens).is_some())
    {
        return None;
    }
    let value = lower_for_property(property, tokens, &mut CssSymbolInterner::new())?;
    let accepted = match (&property, &value) {
        (_, PropertyValue::Keyword(symbol)) => {
            is_style_query_keyword(property) || is_style_query_sizing_keyword(property, *symbol)
        }
        (PropertyId::FontSize, PropertyValue::Length(length)) => !length.is_auto(),
        (_, PropertyValue::Length(_) | PropertyValue::Calc(_)) => {
            property.supports_style_query_length()
        }
        (PropertyId::FlexGrow | PropertyId::FlexShrink, PropertyValue::Number(value)) => {
            value.is_finite() && *value >= 0.0
        }
        (_, PropertyValue::Number(value)) => is_style_query_number(property) && value.is_finite(),
        (_, PropertyValue::Initial)
        | (PropertyId::Color | PropertyId::BackgroundColor, PropertyValue::Color(_)) => true,
        _ => false,
    };
    accepted.then_some(value)
}

/// Maps a feature, comparator, and reference value to a physical or logical axis
/// comparison. Unknown features and unsupported reference lengths return `None`.
fn lower_feature(
    feature: &str,
    comparator: Option<MediaComparator>,
    value: Option<&[ParsedValue]>,
    width_axis: ConditionAxis,
    height_axis: ConditionAxis,
) -> Option<RuleCondition> {
    let (axis, synthesized) = feature_axis(feature, width_axis, height_axis)?;
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

fn feature_axis(
    feature: &str,
    width_axis: ConditionAxis,
    height_axis: ConditionAxis,
) -> Option<(ConditionAxis, Option<ConditionComparator>)> {
    let name = feature.to_ascii_lowercase();
    let inline_axis = if width_axis.is_container() {
        ConditionAxis::ContainerInlineSize
    } else {
        width_axis
    };
    let block_axis = if height_axis.is_container() {
        ConditionAxis::ContainerBlockSize
    } else {
        height_axis
    };
    Some(match name.as_str() {
        "min-width" => (width_axis, Some(ConditionComparator::Ge)),
        "max-width" => (width_axis, Some(ConditionComparator::Le)),
        "width" => (width_axis, None),
        "min-height" => (height_axis, Some(ConditionComparator::Ge)),
        "max-height" => (height_axis, Some(ConditionComparator::Le)),
        "height" => (height_axis, None),
        "min-inline-size" => (inline_axis, Some(ConditionComparator::Ge)),
        "max-inline-size" => (inline_axis, Some(ConditionComparator::Le)),
        "inline-size" => (inline_axis, None),
        "min-block-size" => (block_axis, Some(ConditionComparator::Ge)),
        "max-block-size" => (block_axis, Some(ConditionComparator::Le)),
        "block-size" => (block_axis, None),
        // Unknown features and features handled by media-specific lowering.
        _ => return None,
    })
}

/// Extracts a single `px` reference length (or unitless `0`) from a feature
/// value, returning [`None`] for anything we cannot resolve to pixels.
fn reference_px(value: &[ParsedValue]) -> Option<f32> {
    match value {
        [ParsedValue::Dimension { value, unit }] if unit.eq_ignore_ascii_case("px") => {
            Some(*value as f32)
        }
        // CSS permits a unitless zero length.
        [ParsedValue::Number(number)] if *number == 0.0 => Some(0.0),
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
        assert!(!media_condition("(pointer: coarse)").holds(&env));
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
    fn resolution_accepts_equivalent_units_and_range_bounds() {
        let env = MediaEnvironment::viewport(300.0, 200.0).with_resolution_dppx(2.0);
        for query in [
            "(RESOLUTION: 192dpi)",
            "(min-resolution: 2dppx)",
            "(resolution >= 2x)",
            "(max-resolution: 96dpcm)",
            "(192dpi <= resolution)",
            "(resolution)",
        ] {
            let (condition, unsupported) = lower_media(query);
            assert!(unsupported.is_empty(), "{query}");
            assert!(condition.holds(&env), "{query}");
        }
        assert!(!media_condition("(resolution > 192dpi)").holds(&env));
        assert!(!media_condition("(resolution: -2dppx)").holds(&env));
    }

    #[test]
    fn chained_media_ranges_require_both_bounds() {
        for text in [
            "(1.25dppx <= resolution < 192dpi)",
            "(192dpi > resolution >= 1.25dppx)",
        ] {
            let (condition, unsupported) = lower_media(text);
            assert!(unsupported.is_empty());
            let viewport = MediaEnvironment::viewport(300.0, 200.0);
            for (scale, expected) in [(1.0, false), (1.25, true), (1.75, true), (2.0, false)] {
                assert_eq!(
                    condition.holds(&viewport.with_resolution_dppx(scale)),
                    expected,
                    "{text} at {scale}"
                );
            }
        }
        assert!(
            MediaQueryParser::new()
                .parse(&CssSource::anonymous("(100px < width > 300px)"))
                .is_err()
        );
        assert!(
            MediaQueryParser::new()
                .parse(&CssSource::anonymous("(100px = width < 300px)"))
                .is_err()
        );
    }

    #[test]
    fn orientation_compares_viewport_axes() {
        let (landscape, unsupported) = lower_media("(orientation: landscape)");
        assert!(unsupported.is_empty());
        assert!(landscape.holds(&MediaEnvironment::viewport(300.0, 200.0)));
        assert!(!landscape.holds(&MediaEnvironment::viewport(200.0, 200.0)));
        let portrait = media_condition("(orientation: portrait)");
        assert!(portrait.holds(&MediaEnvironment::viewport(200.0, 200.0)));
        assert!(portrait.holds(&MediaEnvironment::viewport(200.0, 300.0)));
        assert!(!portrait.holds(&MediaEnvironment::unbounded()));
    }

    #[test]
    fn an_unrepresentable_feature_is_reported_as_unsupported() {
        let (_, unsupported) = lower_media("(update: fast)");
        assert_eq!(unsupported.len(), 1);
        assert_eq!(unsupported[0].kind(), UnsupportedFeatureKind::AtRule);
        assert_eq!(unsupported[0].name(), "@media (update)");
    }

    #[test]
    fn every_unrepresentable_feature_in_a_query_is_reported() {
        let (cond, unsupported) = lower_media("(update: fast) and (scan: progressive)");
        assert!(!cond.holds(&MediaEnvironment::viewport(500.0, 500.0)));
        let names: Vec<&str> = unsupported
            .iter()
            .map(UnsupportedCssFeature::name)
            .collect();
        assert_eq!(names, ["@media (update)", "@media (scan)"]);
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
        let (cond, unsupported) = lower_media("not all and (update: fast)");
        assert!(!cond.holds(&MediaEnvironment::viewport(500.0, 500.0)));
        assert_eq!(unsupported.len(), 1);
    }

    #[test]
    fn nested_media_logic_combines_known_and_unknown_conditions() {
        let narrow = MediaEnvironment::viewport(100.0, 500.0);
        let wide = MediaEnvironment::viewport(500.0, 500.0);
        let (condition, unsupported) = lower_media(
            "screen and ((update: fast) or ((width > 200px) and (not (orientation: landscape))))",
        );
        assert_eq!(unsupported.len(), 1);
        assert!(!condition.holds(&narrow));
        assert!(condition.holds(&wide));
        assert!(!media_condition("not ((width <= 200px) or (update: fast))").holds(&wide));
        assert!(media_condition("not ((width <= 200px) and (update: fast))").holds(&wide));
        assert!(!media_condition("speech and (width > 200px)").holds(&wide));
    }

    #[test]
    fn one_unrepresentable_query_does_not_sink_the_list() {
        // A list is a disjunction: the queries we can evaluate still apply.
        let cond = media_condition("(max-width: 100px), (update: fast)");
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
    fn size_container_logic_preserves_unknown_results() {
        let viewport = MediaEnvironment::viewport(1000.0, 1000.0);
        let narrow = viewport.with_container(150.0, 150.0);
        let wide = viewport.with_container(500.0, 500.0);
        let range = container_condition("(width < 200px) or (width > 600px)");
        assert!(range.holds(&narrow));
        assert!(!range.holds(&wide));
        let negated = container_condition("not (width > 200px)");
        assert!(negated.holds(&narrow));
        assert!(!negated.holds(&wide));
        assert!(!negated.holds(&viewport));
        assert!(!container_condition("not (orientation: landscape)").holds(&wide));
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
