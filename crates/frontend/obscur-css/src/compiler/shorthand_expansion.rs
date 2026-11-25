#![doc = include_str!("../../docs/api/compiler/shorthand_expansion.doc/_module.md")]

use super::grid_placement::grid_line_edge;
use super::grid_track_lowering::{lower_grid_shorthand, lower_grid_template_shorthand};
use super::property_compiler::{baseline_position, lower_for_property, value_unsupported_kind};
use super::value_lowering::css_wide_keyword;
use super::{CompiledDeclaration, CssSymbolInterner};
use crate::error::UnsupportedCssFeature;
use crate::parser::{ParsedDeclaration, ParsedValue};
use obscur_identity::InternedSymbol;

use super::custom_value::custom_value_text;
use obscur_style::cascade::{CascadeOrigin, Declaration, ImportantFlag};
use obscur_style::property::{ContainerComponent, GridLineValue, PropertyId, PropertyValue};
use obscur_style::selector::{SelectorId, SelectorSpecificity};

/// Returns the four longhand [`PropertyId`]s (in top/right/bottom/left order) a
/// `margin` / `padding` shorthand expands into, or [`None`] for any other name.
pub(super) fn box_shorthand_sides(name: &str) -> Option<[PropertyId; 4]> {
    if name.eq_ignore_ascii_case("margin") {
        Some([
            PropertyId::MarginTop,
            PropertyId::MarginRight,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
        ])
    } else if name.eq_ignore_ascii_case("padding") {
        Some([
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
        ])
    } else if name.eq_ignore_ascii_case("inset") {
        // The `inset` shorthand fans out into the four positioning longhands in
        // the same top/right/bottom/left order as `margin` / `padding`.
        Some([
            PropertyId::Top,
            PropertyId::Right,
            PropertyId::Bottom,
            PropertyId::Left,
        ])
    } else if name.eq_ignore_ascii_case("border-width") {
        // `border-width` fans its 1-4 length values out to the four border-width
        // longhands, the same edge-replication as `margin` / `padding`.
        Some([
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
        ])
    } else if name.eq_ignore_ascii_case("border-radius") {
        // `border-radius` fans its 1-4 length values out to the four corner
        // longhands in CSS corner order (top-left, top-right, bottom-right,
        // bottom-left). The 1/2/3/4-value replication rule is identical to the
        // box-model one, so the 2-value form `a b` yields the diagonal pairs
        // top-left/bottom-right = `a` and top-right/bottom-left = `b`. Elliptical
        // `/`-separated radii are out of scope (a single radius per corner).
        Some([
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ])
    } else {
        None
    }
}

#[doc = include_str!("../../docs/api/compiler/shorthand_expansion.doc/expand_box_shorthand.md")]
pub(super) fn expand_box_shorthand(
    decl: &ParsedDeclaration,
    sides: [PropertyId; 4],
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
    interner: &mut CssSymbolInterner,
) -> Vec<CompiledDeclaration> {
    let unsupported = || {
        vec![CompiledDeclaration::Unsupported(
            UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
        )]
    };
    if decl.value.len() > 1
        && decl.value.iter().any(
            |value| matches!(value, ParsedValue::Keyword(word) if css_wide_keyword(word).is_some()),
        )
    {
        return unsupported();
    }
    let mut components = Vec::new();
    for value in &decl.value {
        match lower_for_property(sides[0], std::slice::from_ref(value), interner) {
            Some(length) => components.push(length),
            None => return unsupported(),
        }
    }
    let four = match components.as_slice() {
        [a] => [a.clone(), a.clone(), a.clone(), a.clone()],
        [a, b] => [a.clone(), b.clone(), a.clone(), b.clone()],
        [a, b, c] => [a.clone(), b.clone(), c.clone(), b.clone()],
        [a, b, c, d] => [a.clone(), b.clone(), c.clone(), d.clone()],
        _ => return unsupported(),
    };
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    sides
        .into_iter()
        .zip(four)
        .map(|(property, value)| {
            CompiledDeclaration::Standard(Declaration::new(
                property,
                value,
                important,
                origin,
                specificity,
                source_order,
                selector,
            ))
        })
        .collect()
}

/// Returns the two longhand [`PropertyId`]s (`align`, `justify`) that a
/// `place-items` / `place-self` / `place-content` shorthand expands into, or
/// [`None`] for any other name. The CSS `place-*` grammar is `<align> <justify>`,
/// so the *align* side comes first (`place-content: <align-content>
/// <justify-content>`).
pub(super) fn place_shorthand_sides(name: &str) -> Option<(PropertyId, PropertyId)> {
    if name.eq_ignore_ascii_case("place-items") {
        Some((PropertyId::AlignItems, PropertyId::JustifyItems))
    } else if name.eq_ignore_ascii_case("place-self") {
        Some((PropertyId::AlignSelf, PropertyId::JustifySelf))
    } else if name.eq_ignore_ascii_case("place-content") {
        Some((PropertyId::AlignContent, PropertyId::JustifyContent))
    } else {
        None
    }
}

#[doc = include_str!("../../docs/api/compiler/shorthand_expansion.doc/axis_pair_shorthand_sides.md")]
pub(super) fn axis_pair_shorthand_sides(name: &str) -> Option<(PropertyId, PropertyId)> {
    // `overflow: <x> <y>` (css-overflow-3 §3): the inline axis is stated first,
    // and one value sets both.
    if name.eq_ignore_ascii_case("gap") || name.eq_ignore_ascii_case("grid-gap") {
        return Some((PropertyId::RowGap, PropertyId::ColumnGap));
    }
    if name.eq_ignore_ascii_case("contain-intrinsic-size") {
        Some((
            PropertyId::ContainIntrinsicWidth,
            PropertyId::ContainIntrinsicHeight,
        ))
    } else if name.eq_ignore_ascii_case("overscroll-behavior") {
        Some((
            PropertyId::OverscrollBehaviorX,
            PropertyId::OverscrollBehaviorY,
        ))
    } else {
        name.eq_ignore_ascii_case("overflow")
            .then_some((PropertyId::OverflowX, PropertyId::OverflowY))
    }
}

#[doc = include_str!("../../docs/api/compiler/shorthand_expansion.doc/logical_pair_shorthand_sides.md")]
pub(super) fn logical_pair_shorthand_sides(name: &str) -> Option<(PropertyId, PropertyId)> {
    if name.eq_ignore_ascii_case("margin-inline") {
        Some((PropertyId::MarginInlineStart, PropertyId::MarginInlineEnd))
    } else if name.eq_ignore_ascii_case("margin-block") {
        Some((PropertyId::MarginBlockStart, PropertyId::MarginBlockEnd))
    } else if name.eq_ignore_ascii_case("padding-inline") {
        Some((PropertyId::PaddingInlineStart, PropertyId::PaddingInlineEnd))
    } else if name.eq_ignore_ascii_case("padding-block") {
        Some((PropertyId::PaddingBlockStart, PropertyId::PaddingBlockEnd))
    } else if name.eq_ignore_ascii_case("inset-inline") {
        Some((PropertyId::InsetInlineStart, PropertyId::InsetInlineEnd))
    } else if name.eq_ignore_ascii_case("inset-block") {
        Some((PropertyId::InsetBlockStart, PropertyId::InsetBlockEnd))
    } else {
        None
    }
}

#[doc = include_str!("../../docs/api/compiler/shorthand_expansion.doc/expand_pair_shorthand.md")]
#[allow(clippy::too_many_arguments)]
pub(super) fn expand_pair_shorthand(
    decl: &ParsedDeclaration,
    align: PropertyId,
    justify: PropertyId,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
    interner: &mut CssSymbolInterner,
) -> Vec<CompiledDeclaration> {
    let unsupported = || {
        vec![CompiledDeclaration::Unsupported(
            UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
        )]
    };
    let values = if align == PropertyId::RowGap && decl.value.iter().any(contains_var) {
        shorthand_components(&decl.value).map(|components| {
            let value = PropertyValue::GapTemplate(components);
            [value.clone(), value]
        })
    } else {
        pair_values(align, justify, &decl.value, interner)
    };
    let Some([align_value, justify_value]) = values else {
        return unsupported();
    };
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    [(align, align_value), (justify, justify_value)]
        .into_iter()
        .map(|(property, value)| {
            CompiledDeclaration::Standard(Declaration::new(
                property,
                value,
                important,
                origin,
                specificity,
                source_order,
                selector,
            ))
        })
        .collect()
}

/// Lowers a two-longhand shorthand and duplicates a single component when needed.
pub(super) fn pair_values(
    align: PropertyId,
    justify: PropertyId,
    values: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<[PropertyValue; 2]> {
    if matches!(align, PropertyId::ContainIntrinsicWidth | PropertyId::OverscrollBehaviorX | PropertyId::RowGap)
        && values.len() > 1
        && values.iter().any(|value| matches!(value, ParsedValue::Keyword(keyword) if css_wide_keyword(keyword).is_some()))
    {
        return None;
    }
    let item_alignment = matches!(align, PropertyId::AlignItems | PropertyId::AlignSelf);
    let first_is_pair = item_alignment && values.get(..2).and_then(baseline_position).is_some();
    let first_len = if first_is_pair { 2 } else { 1 };
    let Some((align_values, remaining)) = values.split_at_checked(first_len) else {
        return None;
    };
    let justify_values = match remaining {
        [] => align_values,
        [_] => remaining,
        [_, _] if item_alignment && baseline_position(remaining).is_some() => remaining,
        _ => return None,
    };
    if !remaining.is_empty()
        && item_alignment
        && values.iter().any(|value| {
            matches!(value, ParsedValue::Keyword(keyword) if css_wide_keyword(keyword).is_some())
        })
    {
        return None;
    }
    let (Some(align_value), Some(justify_value)) = (
        lower_for_property(align, align_values, interner),
        lower_for_property(justify, justify_values, interner),
    ) else {
        return None;
    };
    Some([align_value, justify_value])
}

/// Expands a supported container shorthand into the two longhands with the same cascade metadata. An omitted type resets to its initial value.
pub(super) fn expand_container_shorthand(
    decl: &ParsedDeclaration,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
    interner: &mut CssSymbolInterner,
) -> Vec<CompiledDeclaration> {
    let Some((name, kind)) = deferred_container(&decl.value)
        .map(|reference| (reference.clone(), reference))
        .or_else(|| container_values(&decl.value, interner))
    else {
        return vec![CompiledDeclaration::Unsupported(
            UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
        )];
    };
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    [
        (PropertyId::ContainerName, name),
        (PropertyId::ContainerType, kind),
    ]
    .into_iter()
    .map(|(property, value)| {
        CompiledDeclaration::Standard(Declaration::new(
            property,
            value,
            important,
            origin,
            specificity,
            source_order,
            selector,
        ))
    })
    .collect()
}

fn deferred_container(values: &[ParsedValue]) -> Option<PropertyValue> {
    if let [value] = values
        && let ContainerComponent::Variable { name, fallback } = container_reference(value)?
    {
        return Some(PropertyValue::ContainerRef { name, fallback });
    }
    shorthand_components(values).map(PropertyValue::ContainerTemplate)
}

pub(super) fn shorthand_components(
    values: &[ParsedValue],
) -> Option<std::sync::Arc<[ContainerComponent]>> {
    if !values.iter().any(contains_var) {
        return None;
    }
    let components = values
        .iter()
        .map(|value| match value {
            value if is_var(value) => container_reference(value),
            ParsedValue::Function { name, args } if args.iter().any(contains_var) => {
                Some(ContainerComponent::Function {
                    name: name.clone(),
                    args: shorthand_components(args)?,
                })
            }
            value => Some(ContainerComponent::Literal(custom_value_text(
                std::slice::from_ref(value),
            ))),
        })
        .collect::<Option<Vec<_>>>()?;
    Some(components.into())
}

fn is_var(value: &ParsedValue) -> bool {
    matches!(value, ParsedValue::Function { name, .. } if name.eq_ignore_ascii_case("var"))
}

pub(super) fn contains_var(value: &ParsedValue) -> bool {
    match value {
        ParsedValue::Function { args, .. } => is_var(value) || args.iter().any(contains_var),
        _ => false,
    }
}

pub(super) fn container_reference(value: &ParsedValue) -> Option<ContainerComponent> {
    let ParsedValue::Function { args, .. } = value else {
        return None;
    };
    if !is_var(value) {
        return None;
    }
    let [ParsedValue::Keyword(name), rest @ ..] = args.as_slice() else {
        return None;
    };
    if !name.starts_with("--") || name.len() == 2 {
        return None;
    }
    let name = InternedSymbol::named(name);
    match rest {
        [] => Some(ContainerComponent::Variable {
            name,
            fallback: None,
        }),
        [ParsedValue::Comma, fallback @ ..] if fallback.iter().any(contains_var) => {
            Some(ContainerComponent::VariableWithTemplateFallback {
                name,
                fallback: shorthand_components(fallback)?,
            })
        }
        [ParsedValue::Comma, fallback @ ..] => Some(ContainerComponent::Variable {
            name,
            fallback: Some(custom_value_text(fallback).into()),
        }),
        _ => None,
    }
}

pub(super) fn container_values(
    values: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<(PropertyValue, PropertyValue)> {
    if let [ParsedValue::Keyword(keyword)] = values
        && let Some(wide) = css_wide_keyword(keyword)
    {
        return Some((wide.clone(), wide));
    }
    let mut parts = values.split(|value| matches!(value, ParsedValue::Delimiter('/')));
    let names = parts.next()?;
    let kind = match parts.next() {
        None => PropertyValue::Initial,
        Some([token @ ParsedValue::Keyword(keyword)]) if css_wide_keyword(keyword).is_none() => {
            lower_for_property(
                PropertyId::ContainerType,
                std::slice::from_ref(token),
                interner,
            )?
        }
        _ => return None,
    };
    if parts.next().is_some()
        || !names.iter().all(
            |value| matches!(value, ParsedValue::Keyword(name) if css_wide_keyword(name).is_none()),
        )
    {
        return None;
    }
    let names = lower_for_property(PropertyId::ContainerName, names, interner)?;
    Some((names, kind))
}

/// Expands flex-flow into its direction and wrapping longhands.
pub(super) fn expand_flex_flow_shorthand(
    decl: &ParsedDeclaration,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
) -> Vec<CompiledDeclaration> {
    let values = if decl.value.iter().any(is_var) {
        shorthand_components(&decl.value).map(|components| {
            let deferred = PropertyValue::FlexFlowTemplate(components);
            [deferred.clone(), deferred]
        })
    } else {
        flex_flow_values(&decl.value)
    };
    let Some([direction, wrap]) = values else {
        return vec![CompiledDeclaration::Unsupported(
            UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
        )];
    };
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    [
        (PropertyId::FlexDirection, direction),
        (PropertyId::FlexWrap, wrap),
    ]
    .into_iter()
    .map(|(property, value)| {
        CompiledDeclaration::Standard(Declaration::new(
            property,
            value,
            important,
            origin,
            specificity,
            source_order,
            selector,
        ))
    })
    .collect()
}

/// Parses direction and wrapping in either order, filling omitted components with their initial values.
pub(super) fn flex_flow_values(values: &[ParsedValue]) -> Option<[PropertyValue; 2]> {
    use obscur_style::property::layout_keyword::{flex_direction_from_name, flex_wrap_from_name};

    if let [ParsedValue::Keyword(keyword)] = values
        && let Some(wide) = css_wide_keyword(keyword)
    {
        return Some([wide.clone(), wide]);
    }
    if !(1..=2).contains(&values.len()) {
        return None;
    }
    let mut direction = None;
    let mut wrap = None;
    for value in values {
        let ParsedValue::Keyword(keyword) = value else {
            return None;
        };
        if let Some(value) = flex_direction_from_name(keyword) {
            if direction.replace(value).is_some() {
                return None;
            }
        } else if let Some(value) = flex_wrap_from_name(keyword) {
            if wrap.replace(value).is_some() {
                return None;
            }
        } else {
            return None;
        }
    }
    Some([
        direction.map_or(PropertyValue::Initial, PropertyValue::Keyword),
        wrap.map_or(PropertyValue::Initial, PropertyValue::Keyword),
    ])
}

#[doc = include_str!("../../docs/api/compiler/shorthand_expansion.doc/expand_flex_shorthand.md")]
pub(super) fn expand_flex_shorthand(
    decl: &ParsedDeclaration,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
    interner: &mut CssSymbolInterner,
) -> Vec<CompiledDeclaration> {
    let unsupported = || {
        vec![CompiledDeclaration::Unsupported(
            UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
        )]
    };
    // A component that is a `<number [0,∞]>`, i.e. a flex-factor candidate.
    let factor = |value: &ParsedValue| match value {
        ParsedValue::Number(n) if *n >= 0.0 => Some(*n),
        _ => None,
    };
    let number = |n: f64| vec![ParsedValue::Number(n)];
    let zero_basis = || {
        vec![ParsedValue::Dimension {
            value: 0.0,
            unit: "px".into(),
        }]
    };
    let one = || number(1.0);

    // The three longhands' component slices, defaults already filled in.
    let (grow, shrink, basis): (Vec<ParsedValue>, Vec<ParsedValue>, Vec<ParsedValue>) = match &*decl
        .value
    {
        // `none` → `0 0 auto`; a CSS-wide keyword goes to all three longhands.
        [ParsedValue::Keyword(keyword)] if keyword.eq_ignore_ascii_case("none") => (
            number(0.0),
            number(0.0),
            vec![ParsedValue::Keyword("auto".into())],
        ),
        [value @ ParsedValue::Keyword(keyword)] if css_wide_keyword(keyword).is_some() => {
            let wide = vec![value.clone()];
            (wide.clone(), wide.clone(), wide)
        }
        // One value: a lone factor (basis `0px`) or a lone basis (factors `1`).
        [value] => match factor(value) {
            Some(grow) => (number(grow), one(), zero_basis()),
            None => (one(), one(), vec![value.clone()]),
        },
        // Two values, in either `||` order.
        [first, second] => match (factor(first), factor(second)) {
            (Some(grow), Some(shrink)) => (number(grow), number(shrink), zero_basis()),
            (Some(grow), None) => (number(grow), one(), vec![second.clone()]),
            (None, Some(grow)) => (number(grow), one(), vec![first.clone()]),
            (None, None) => return unsupported(),
        },
        // Three values: the two factors adjacent, the basis on either end.
        [first, second, third] => match (factor(first), factor(second), factor(third)) {
            // `<grow> <shrink> <basis>`; as a basis the only number is `0`.
            (Some(grow), Some(shrink), _) => {
                if matches!(third, ParsedValue::Number(n) if *n != 0.0) {
                    return unsupported();
                }
                let basis = if matches!(third, ParsedValue::Number(_)) {
                    zero_basis()
                } else {
                    vec![third.clone()]
                };
                (number(grow), number(shrink), basis)
            }
            // `<basis> <grow> <shrink>`.
            (None, Some(grow), Some(shrink)) => (number(grow), number(shrink), vec![first.clone()]),
            _ => return unsupported(),
        },
        _ => return unsupported(),
    };

    let lowered = [
        (PropertyId::FlexGrow, grow),
        (PropertyId::FlexShrink, shrink),
        (PropertyId::FlexBasis, basis),
    ]
    .into_iter()
    .map(|(property, values)| {
        lower_for_property(property, &values, interner).map(|value| (property, value))
    })
    .collect::<Option<Vec<_>>>();
    let Some(lowered) = lowered else {
        return unsupported();
    };
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    lowered
        .into_iter()
        .map(|(property, value)| {
            CompiledDeclaration::Standard(Declaration::new(
                property,
                value,
                important,
                origin,
                specificity,
                source_order,
                selector,
            ))
        })
        .collect()
}

/// Returns the two longhand [`PropertyId`]s (`start`, `end`) that the
/// `grid-column` / `grid-row` shorthand expands into, or [`None`] for any other
/// name.
pub(super) fn grid_line_shorthand_sides(name: &str) -> Option<(PropertyId, PropertyId)> {
    if name.eq_ignore_ascii_case("grid-column") {
        Some((PropertyId::GridColumnStart, PropertyId::GridColumnEnd))
    } else if name.eq_ignore_ascii_case("grid-row") {
        Some((PropertyId::GridRowStart, PropertyId::GridRowEnd))
    } else {
        None
    }
}

/// Expands grid or grid-template atomically into its longhands.
pub(super) fn expand_grid_shorthand(
    decl: &ParsedDeclaration,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
) -> Vec<CompiledDeclaration> {
    let values = if decl.name.eq_ignore_ascii_case("grid") {
        lower_grid_shorthand(&decl.value).map(Vec::from)
    } else {
        lower_grid_template_shorthand(&decl.value).map(Vec::from)
    };
    let Some(values) = values else {
        return vec![CompiledDeclaration::Unsupported(
            UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
        )];
    };
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    [
        PropertyId::GridTemplateRows,
        PropertyId::GridTemplateColumns,
        PropertyId::GridTemplateAreas,
        PropertyId::GridAutoRows,
        PropertyId::GridAutoColumns,
        PropertyId::GridAutoFlow,
    ]
    .into_iter()
    .zip(values)
    .map(|(property, value)| {
        CompiledDeclaration::Standard(Declaration::new(
            property,
            value,
            important,
            origin,
            specificity,
            source_order,
            selector,
        ))
    })
    .collect()
}

#[doc = include_str!("../../docs/api/compiler/shorthand_expansion.doc/expand_grid_line_shorthand.md")]
pub(super) fn expand_grid_line_shorthand(
    decl: &ParsedDeclaration,
    start_id: PropertyId,
    end_id: PropertyId,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
) -> Vec<CompiledDeclaration> {
    let unsupported = || {
        vec![CompiledDeclaration::Unsupported(
            UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
        )]
    };
    // A `/` separates the start edge from the end edge; more than one `/` is not a
    // valid two-value placement.
    let mut sides = decl
        .value
        .split(|v| matches!(v, ParsedValue::Delimiter('/')));
    let Some(start_group) = sides.next() else {
        return unsupported();
    };
    let end_group = sides.next();
    if sides.next().is_some() {
        return unsupported();
    }
    let Some(start) = grid_line_edge(start_group) else {
        return unsupported();
    };
    // A bare name applies to both edges when the end is omitted.
    let end = match end_group {
        Some(group) => match grid_line_edge(group) {
            Some(edge) => edge,
            None => return unsupported(),
        },
        None => omitted_grid_edge(&start),
    };
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    [(start_id, start), (end_id, end)]
        .into_iter()
        .map(|(property, edge)| {
            CompiledDeclaration::Standard(Declaration::new(
                property,
                PropertyValue::GridLine(edge),
                important,
                origin,
                specificity,
                source_order,
                selector,
            ))
        })
        .collect()
}

/// Expands a grid-area declaration into row-start, column-start, row-end and
/// column-end. An omitted edge copies its corresponding bare name or uses
/// auto. CSS-wide keywords apply to all four properties. A named area is
/// resolved by the same line lookup as grid-row and grid-column.
pub(super) fn expand_grid_area_shorthand(
    decl: &ParsedDeclaration,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
) -> Vec<CompiledDeclaration> {
    let unsupported = || {
        vec![CompiledDeclaration::Unsupported(
            UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
        )]
    };
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    let values = if let [ParsedValue::Keyword(keyword)] = decl.value.as_slice()
        && let Some(wide) = css_wide_keyword(keyword)
    {
        std::array::from_fn(|_| wide.clone())
    } else {
        let mut edges = decl
            .value
            .split(|value| matches!(value, ParsedValue::Delimiter('/')));
        let Some(row_start) = edges.next().and_then(grid_line_edge) else {
            return unsupported();
        };
        let mut take_edge = |fallback: &GridLineValue| match edges.next() {
            Some(group) => grid_line_edge(group),
            None => Some(omitted_grid_edge(fallback)),
        };
        let Some(column_start) = take_edge(&row_start) else {
            return unsupported();
        };
        let Some(row_end) = take_edge(&row_start) else {
            return unsupported();
        };
        let Some(column_end) = take_edge(&column_start) else {
            return unsupported();
        };
        if edges.next().is_some() {
            return unsupported();
        }
        [row_start, column_start, row_end, column_end].map(PropertyValue::GridLine)
    };
    [
        PropertyId::GridRowStart,
        PropertyId::GridColumnStart,
        PropertyId::GridRowEnd,
        PropertyId::GridColumnEnd,
    ]
    .into_iter()
    .zip(values)
    .map(|(property, value)| {
        CompiledDeclaration::Standard(Declaration::new(
            property,
            value,
            important,
            origin,
            specificity,
            source_order,
            selector,
        ))
    })
    .collect()
}

/// Only a bare custom identifier is copied to an omitted grid edge.
fn omitted_grid_edge(start: &GridLineValue) -> GridLineValue {
    if matches!(start, GridLineValue::Named(_)) {
        start.clone()
    } else {
        GridLineValue::Auto
    }
}

#[cfg(test)]
mod tests {
    use super::super::grid_placement::tests::grid_lines;
    use super::super::longhand_catalog::property_id_from_name;
    use super::super::property_compiler::tests::{compile_all, pairs, standard_of, value_of};

    use crate::compiler::CompiledDeclaration;

    use obscur_identity::InternedSymbol;
    use obscur_style::property::{
        GridLineValue, Length, PropertyId, PropertyValue, layout_keyword,
    };

    #[test]
    fn contain_intrinsic_size_expands_lengths_and_rejects_invalid_components() {
        let width = PropertyId::ContainIntrinsicWidth;
        let height = PropertyId::ContainIntrinsicHeight;
        assert_eq!(
            pairs(&compile_all("contain-intrinsic-size: 20px none")),
            [
                (width, PropertyValue::Length(Length::px(20.0))),
                (height, PropertyValue::Initial)
            ]
        );
        assert_eq!(
            pairs(&compile_all("contain-intrinsic-size: 0")),
            [
                (width, PropertyValue::Length(Length::px(0.0))),
                (height, PropertyValue::Length(Length::px(0.0)))
            ]
        );
        for css in [
            "contain-intrinsic-size: 20px -1px",
            "contain-intrinsic-size: 10%",
            "contain-intrinsic-size: calc(1px + 5%)",
            "contain-intrinsic-size: inherit 20px",
            "contain-intrinsic-size: none none none",
        ] {
            assert!(
                matches!(
                    compile_all(css).as_slice(),
                    [CompiledDeclaration::Unsupported(_)]
                ),
                "{css}"
            );
        }
    }

    #[test]
    fn both_container_declarations_accept_multiple_names() {
        let names = PropertyValue::ContainerNames(vec!["Layout".into(), "Wide".into()].into());
        assert_eq!(
            pairs(&compile_all("container-name: Layout Wide")),
            [(PropertyId::ContainerName, names.clone())]
        );
        assert_eq!(
            pairs(&compile_all("container: Layout Wide / size")),
            [
                (PropertyId::ContainerName, names),
                (
                    PropertyId::ContainerType,
                    PropertyValue::Name("size".into())
                )
            ]
        );
        for css in [
            "container-name: none Wide",
            "container-name: Layout, Wide",
            "container: Layout inherit / size",
        ] {
            assert!(
                matches!(
                    compile_all(css).as_slice(),
                    [CompiledDeclaration::Unsupported(_)]
                ),
                "{css}"
            );
        }
    }

    #[test]
    fn container_shorthand_resets_omitted_type_and_rejects_invalid_pairs() {
        let name = PropertyId::ContainerName;
        let kind = PropertyId::ContainerType;
        assert_eq!(
            pairs(&compile_all("container: Sidebar/INLINE-SIZE")),
            [
                (name, PropertyValue::Name("Sidebar".into())),
                (kind, PropertyValue::Name("inline-size".into()))
            ]
        );
        assert_eq!(
            pairs(&compile_all("container: sidebar")),
            [
                (name, PropertyValue::Name("sidebar".into())),
                (kind, PropertyValue::Initial)
            ]
        );
        assert_eq!(
            pairs(&compile_all("container: inherit")),
            [
                (name, PropertyValue::Inherit),
                (kind, PropertyValue::Inherit)
            ]
        );
        for css in [
            "container: sidebar /",
            "container: / size",
            "container: sidebar 2px",
            "container: sidebar / inherit",
            "container: unset / size",
            "container: sidebar / auto",
            "container: sidebar / size / size",
            "container: not / size",
        ] {
            assert!(
                matches!(
                    compile_all(css).as_slice(),
                    [CompiledDeclaration::Unsupported(_)]
                ),
                "{css}"
            );
        }
    }

    #[test]
    fn a_var_in_one_side_leaves_the_other_sides_standing() {
        // Consumer audit 2-C: one unresolvable component used to degrade the
        // WHOLE declaration, so the literal zero was lost along with the gap.
        let gap = PropertyValue::CustomRef(InternedSymbol::named("--k-gap"));
        assert_eq!(
            pairs(&compile_all("margin: 0 var(--k-gap)")),
            vec![
                (
                    PropertyId::MarginTop,
                    PropertyValue::Length(Length::px(0.0))
                ),
                (PropertyId::MarginRight, gap.clone()),
                (
                    PropertyId::MarginBottom,
                    PropertyValue::Length(Length::px(0.0))
                ),
                (PropertyId::MarginLeft, gap),
            ]
        );
    }

    /// A math function is a `<length>` (CSS Values 4 §10.1), so a box shorthand
    /// takes one. It used to reject the component, which does not degrade one
    /// corner — it degrades the whole declaration, leaving square corners with no
    /// diagnostic anywhere.
    #[test]
    fn a_box_shorthand_takes_a_math_function_per_side() {
        let compiled = pairs(&compile_all("border-radius: min(3vmin, 14px)"));
        assert_eq!(compiled.len(), 4, "all four corners are emitted");
        assert!(
            compiled
                .iter()
                .all(|(_, value)| matches!(value, PropertyValue::Calc(_))),
            "each corner carries the expression, not a dropped declaration"
        );
    }

    /// The same for the box-model shorthands, which share the component rule.
    #[test]
    fn padding_takes_a_math_function_per_side() {
        let compiled = pairs(&compile_all("padding: calc(2px + 1em) 4px"));
        assert_eq!(compiled.len(), 4);
        assert!(matches!(compiled[0].1, PropertyValue::Calc(_)));
        assert!(matches!(
            compiled[1].1,
            PropertyValue::Length(Length::Px(_))
        ));
    }

    #[test]
    fn border_radius_one_value_rounds_all_four_corners() {
        let r = PropertyValue::Length(Length::px(8.0));
        assert_eq!(
            pairs(&compile_all("border-radius: 8px")),
            vec![
                (PropertyId::BorderTopLeftRadius, r.clone()),
                (PropertyId::BorderTopRightRadius, r.clone()),
                (PropertyId::BorderBottomRightRadius, r.clone()),
                (PropertyId::BorderBottomLeftRadius, r),
            ]
        );
    }

    #[test]
    fn border_radius_two_value_sets_diagonal_pairs() {
        // `4px 8px` → top-left/bottom-right = 4, top-right/bottom-left = 8.
        let four = PropertyValue::Length(Length::px(4.0));
        let eight = PropertyValue::Length(Length::px(8.0));
        assert_eq!(
            pairs(&compile_all("border-radius: 4px 8px")),
            vec![
                (PropertyId::BorderTopLeftRadius, four.clone()),
                (PropertyId::BorderTopRightRadius, eight.clone()),
                (PropertyId::BorderBottomRightRadius, four),
                (PropertyId::BorderBottomLeftRadius, eight),
            ]
        );
    }

    #[test]
    fn border_radius_four_value_sets_each_corner() {
        let got = pairs(&compile_all("border-radius: 1px 2px 3px 4px"));
        assert_eq!(
            got,
            vec![
                (
                    PropertyId::BorderTopLeftRadius,
                    PropertyValue::Length(Length::px(1.0))
                ),
                (
                    PropertyId::BorderTopRightRadius,
                    PropertyValue::Length(Length::px(2.0))
                ),
                (
                    PropertyId::BorderBottomRightRadius,
                    PropertyValue::Length(Length::px(3.0))
                ),
                (
                    PropertyId::BorderBottomLeftRadius,
                    PropertyValue::Length(Length::px(4.0))
                ),
            ]
        );
    }

    #[test]
    fn per_corner_radius_longhand_lowers_to_a_length() {
        let got = pairs(&compile_all("border-top-left-radius: 6px"));
        assert_eq!(
            got,
            vec![(
                PropertyId::BorderTopLeftRadius,
                PropertyValue::Length(Length::px(6.0))
            )]
        );
        assert_eq!(
            property_id_from_name("border-bottom-right-radius"),
            Some(PropertyId::BorderBottomRightRadius)
        );
    }

    /// The `(property, value)` pairs a shorthand compiles to, in order.
    fn pairs_of(css: &str) -> Vec<(PropertyId, PropertyValue)> {
        compile_all(css)
            .iter()
            .map(|c| (standard_of(c).property(), standard_of(c).value().clone()))
            .collect()
    }

    #[test]
    fn a_logical_axis_shorthand_expands_start_then_end() {
        assert_eq!(
            pairs_of("margin-inline: 10px 20px"),
            vec![
                (
                    PropertyId::MarginInlineStart,
                    PropertyValue::Length(Length::px(10.0))
                ),
                (
                    PropertyId::MarginInlineEnd,
                    PropertyValue::Length(Length::px(20.0))
                ),
            ],
            "the first component is the START edge"
        );
    }

    #[test]
    fn a_logical_axis_shorthand_replicates_a_single_value_to_both_edges() {
        assert_eq!(
            pairs_of("padding-block: 8px"),
            vec![
                (
                    PropertyId::PaddingBlockStart,
                    PropertyValue::Length(Length::px(8.0))
                ),
                (
                    PropertyId::PaddingBlockEnd,
                    PropertyValue::Length(Length::px(8.0))
                ),
            ]
        );
    }

    #[test]
    fn every_logical_axis_shorthand_reaches_its_own_pair() {
        // Each of the six names must land on its OWN two longhands — a copy-paste
        // slip in the table would otherwise send `padding-inline` to the margins.
        let cases = [
            (
                "margin-inline: 1px",
                [PropertyId::MarginInlineStart, PropertyId::MarginInlineEnd],
            ),
            (
                "margin-block: 1px",
                [PropertyId::MarginBlockStart, PropertyId::MarginBlockEnd],
            ),
            (
                "padding-inline: 1px",
                [PropertyId::PaddingInlineStart, PropertyId::PaddingInlineEnd],
            ),
            (
                "padding-block: 1px",
                [PropertyId::PaddingBlockStart, PropertyId::PaddingBlockEnd],
            ),
            (
                "inset-inline: 1px",
                [PropertyId::InsetInlineStart, PropertyId::InsetInlineEnd],
            ),
            (
                "inset-block: 1px",
                [PropertyId::InsetBlockStart, PropertyId::InsetBlockEnd],
            ),
        ];
        for (css, expected) in cases {
            let ids: Vec<_> = compile_all(css)
                .iter()
                .map(|c| standard_of(c).property())
                .collect();
            assert_eq!(ids, expected.to_vec(), "for `{css}`");
        }
    }

    #[test]
    fn a_logical_axis_shorthand_accepts_the_units_its_longhands_do() {
        // Lowered through each longhand's own rules, so relative units and a
        // unitless zero work exactly as on the physical edges.
        assert_eq!(
            pairs_of("margin-inline: 2em 0"),
            vec![
                (
                    PropertyId::MarginInlineStart,
                    PropertyValue::Length(Length::Em(2.0))
                ),
                (
                    PropertyId::MarginInlineEnd,
                    PropertyValue::Length(Length::px(0.0))
                ),
            ]
        );
    }

    #[test]
    fn a_logical_axis_shorthand_with_too_many_components_is_unsupported() {
        // `margin-inline` names one axis: two edges, never four.
        let out = compile_all("margin-inline: 1px 2px 3px");
        assert!(
            matches!(out.as_slice(), [CompiledDeclaration::Unsupported(_)]),
            "expected one unsupported declaration, got {out:?}"
        );
    }

    #[test]
    fn the_physical_box_shorthands_still_take_four_values() {
        // The regression guard: adding the axis pair must not have captured
        // `margin` / `padding` themselves, which fan out to four edges.
        let ids: Vec<_> = compile_all("margin: 1px 2px 3px 4px")
            .iter()
            .map(|c| standard_of(c).property())
            .collect();
        assert_eq!(
            ids,
            vec![
                PropertyId::MarginTop,
                PropertyId::MarginRight,
                PropertyId::MarginBottom,
                PropertyId::MarginLeft,
            ]
        );
    }

    #[test]
    fn place_items_expands_align_then_justify() {
        // `place-items: <align> <justify>` — align side first.
        let out = compile_all("place-items: center start");
        let ids: Vec<_> = out
            .iter()
            .map(|c| (standard_of(c).property(), standard_of(c).value().clone()))
            .collect();
        assert_eq!(
            ids,
            vec![
                (
                    PropertyId::AlignItems,
                    PropertyValue::Keyword(layout_keyword::ALIGN_CENTER)
                ),
                (
                    PropertyId::JustifyItems,
                    PropertyValue::Keyword(layout_keyword::ALIGN_START)
                ),
            ]
        );
    }

    #[test]
    fn place_items_single_value_replicates_to_both_axes() {
        let out = compile_all("place-items: stretch");
        let ids: Vec<_> = out.iter().map(|c| standard_of(c).property()).collect();
        assert_eq!(ids, vec![PropertyId::AlignItems, PropertyId::JustifyItems]);
        for c in &out {
            assert_eq!(
                standard_of(c).value(),
                &PropertyValue::Keyword(layout_keyword::ALIGN_STRETCH)
            );
        }
    }

    #[test]
    fn place_self_expands_and_accepts_auto() {
        let out = compile_all("place-self: end auto");
        let ids: Vec<_> = out
            .iter()
            .map(|c| (standard_of(c).property(), standard_of(c).value().clone()))
            .collect();
        assert_eq!(
            ids,
            vec![
                (
                    PropertyId::AlignSelf,
                    PropertyValue::Keyword(layout_keyword::ALIGN_END)
                ),
                (
                    PropertyId::JustifySelf,
                    PropertyValue::Keyword(layout_keyword::ALIGN_AUTO)
                ),
            ]
        );
    }

    #[test]
    fn place_alignment_keeps_two_word_baseline_values_together() {
        use layout_keyword::{ALIGN_BASELINE, ALIGN_CENTER, ALIGN_LAST_BASELINE};

        for (css, expected) in [
            (
                "place-items: last baseline",
                [ALIGN_LAST_BASELINE, ALIGN_LAST_BASELINE],
            ),
            (
                "place-items: first baseline center",
                [ALIGN_BASELINE, ALIGN_CENTER],
            ),
            (
                "place-self: center baseline last",
                [ALIGN_CENTER, ALIGN_LAST_BASELINE],
            ),
            (
                "place-self: baseline first last baseline",
                [ALIGN_BASELINE, ALIGN_LAST_BASELINE],
            ),
        ] {
            let output = compile_all(css);
            let values: Vec<_> = output
                .iter()
                .map(|item| standard_of(item).value().clone())
                .collect();
            assert_eq!(values, expected.map(PropertyValue::Keyword), "{css}");
        }

        for css in [
            "place-self: last baseline center end",
            "place-items: last baseline inherit",
            "place-items: first center",
        ] {
            assert!(
                matches!(&compile_all(css)[..], [CompiledDeclaration::Unsupported(_)]),
                "{css}"
            );
        }
    }

    #[test]
    fn place_content_expands_align_then_justify() {
        // `place-content: <align-content> <justify-content>` — align side first.
        let out = compile_all("place-content: center end");
        let ids: Vec<_> = out
            .iter()
            .map(|c| (standard_of(c).property(), standard_of(c).value().clone()))
            .collect();
        assert_eq!(
            ids,
            vec![
                (
                    PropertyId::AlignContent,
                    PropertyValue::Keyword(layout_keyword::JUSTIFY_CENTER)
                ),
                (
                    PropertyId::JustifyContent,
                    PropertyValue::Keyword(layout_keyword::JUSTIFY_END)
                ),
            ]
        );
    }

    #[test]
    fn overflow_shorthand_expands_to_both_axes_and_the_longhands_stand_alone() {
        // `overflow: <x> <y>`, inline axis first (css-overflow-3 §3).
        let two = compile_all("overflow: hidden auto");
        let ids: Vec<_> = two
            .iter()
            .map(|c| (standard_of(c).property(), standard_of(c).value().clone()))
            .collect();
        assert_eq!(
            ids,
            vec![
                (
                    PropertyId::OverflowX,
                    PropertyValue::Keyword(layout_keyword::OVERFLOW_HIDDEN)
                ),
                (
                    PropertyId::OverflowY,
                    PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO)
                ),
            ]
        );

        // One value sets both axes.
        let one = compile_all("overflow: auto");
        assert_eq!(
            one.iter()
                .map(|c| (standard_of(c).property(), standard_of(c).value().clone()))
                .collect::<Vec<_>>(),
            vec![
                (
                    PropertyId::OverflowX,
                    PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO)
                ),
                (
                    PropertyId::OverflowY,
                    PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO)
                ),
            ]
        );

        // And a lone axis longhand compiles to that axis and nothing else — the
        // declaration an author writes to scroll one direction, which used to
        // resolve to no property at all and be dropped as unsupported.
        let axis = compile_all("overflow-y: auto");
        assert_eq!(
            axis.iter()
                .map(|c| (standard_of(c).property(), standard_of(c).value().clone()))
                .collect::<Vec<_>>(),
            vec![(
                PropertyId::OverflowY,
                PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO)
            )]
        );
    }

    #[test]
    fn overflow_as_one_var_reference_reaches_both_axes() {
        // The case nobody had pinned when `overflow` became a pair shorthand.
        // A value that is one `var()` is carried, not judged — so it lands on
        // BOTH axes, and each axis makes its own computed-value-time judgement
        // once the cascade knows what the name stands for. Sinking it here
        // instead would be the 2-C defect again, one shorthand further along.
        let decls = compile_all("overflow: var(--k-scroll)");
        let scroll = PropertyValue::CustomRef(InternedSymbol::named("--k-scroll"));
        assert_eq!(
            decls
                .iter()
                .map(|c| (standard_of(c).property(), standard_of(c).value().clone()))
                .collect::<Vec<_>>(),
            vec![
                (PropertyId::OverflowX, scroll.clone()),
                (PropertyId::OverflowY, scroll),
            ]
        );
    }

    #[test]
    fn inset_shorthand_expands_to_four_inset_longhands() {
        // `inset: 4px 8px` => top/bottom = 4px, right/left = 8px.
        let decls = compile_all("inset: 4px 8px");
        assert_eq!(decls.len(), 4);
        let ids: Vec<_> = decls.iter().map(|d| standard_of(d).property()).collect();
        assert_eq!(
            ids,
            vec![
                PropertyId::Top,
                PropertyId::Right,
                PropertyId::Bottom,
                PropertyId::Left,
            ]
        );
        let value = |i: usize| standard_of(&decls[i]).value().clone();
        assert_eq!(value(0), PropertyValue::Length(Length::px(4.0))); // top
        assert_eq!(value(1), PropertyValue::Length(Length::px(8.0))); // right
        assert_eq!(value(2), PropertyValue::Length(Length::px(4.0))); // bottom
        assert_eq!(value(3), PropertyValue::Length(Length::px(8.0))); // left
    }

    #[test]
    fn margin_shorthand_expands_to_four_longhands() {
        let decls = compile_all("margin: 8px");
        assert_eq!(decls.len(), 4);
        let ids: Vec<_> = decls.iter().map(|d| standard_of(d).property()).collect();
        assert_eq!(
            ids,
            vec![
                PropertyId::MarginTop,
                PropertyId::MarginRight,
                PropertyId::MarginBottom,
                PropertyId::MarginLeft,
            ]
        );
        for d in &decls {
            assert_eq!(
                *standard_of(d).value(),
                PropertyValue::Length(Length::px(8.0))
            );
        }
    }

    #[test]
    fn padding_two_value_shorthand_maps_sides() {
        let decls = compile_all("padding: 4px 8px");
        assert_eq!(decls.len(), 4);
        let value = |i: usize| standard_of(&decls[i]).value().clone();
        // top/bottom = 4px, right/left = 8px.
        assert_eq!(value(0), PropertyValue::Length(Length::px(4.0)));
        assert_eq!(value(1), PropertyValue::Length(Length::px(8.0)));
        assert_eq!(value(2), PropertyValue::Length(Length::px(4.0)));
        assert_eq!(value(3), PropertyValue::Length(Length::px(8.0)));
        assert_eq!(standard_of(&decls[3]).property(), PropertyId::PaddingLeft);
    }

    #[test]
    fn four_value_margin_shorthand_maps_each_side() {
        let decls = compile_all("margin: 1px 2px 3px 4px");
        let value = |i: usize| standard_of(&decls[i]).value().clone();
        assert_eq!(value(0), PropertyValue::Length(Length::px(1.0)));
        assert_eq!(value(1), PropertyValue::Length(Length::px(2.0)));
        assert_eq!(value(2), PropertyValue::Length(Length::px(3.0)));
        assert_eq!(value(3), PropertyValue::Length(Length::px(4.0)));
    }

    #[test]
    fn unitless_zero_shorthand_lowers_to_zero_px() {
        let decls = compile_all("margin: 0");
        assert_eq!(decls.len(), 4);
        assert_eq!(
            *standard_of(&decls[0]).value(),
            PropertyValue::Length(Length::px(0.0))
        );
    }

    #[test]
    fn non_length_shorthand_is_unsupported() {
        let decls = compile_all("margin: red");
        assert_eq!(decls.len(), 1);
        assert!(matches!(decls[0], CompiledDeclaration::Unsupported(_)));
    }

    #[test]
    fn grid_column_named_lines_lower_to_named_edges() {
        // `grid-column: b / c` -> start = Named("b"), end = Named("c").
        assert_eq!(
            grid_lines(&compile_all("grid-column: b / c")),
            vec![
                (
                    PropertyId::GridColumnStart,
                    GridLineValue::Named("b".into())
                ),
                (PropertyId::GridColumnEnd, GridLineValue::Named("c".into())),
            ]
        );
    }

    #[test]
    fn grid_column_single_named_line_sets_both_edges() {
        // The bare name is used for both edges.
        assert_eq!(
            grid_lines(&compile_all("grid-column: main")),
            vec![
                (
                    PropertyId::GridColumnStart,
                    GridLineValue::Named("main".into())
                ),
                (
                    PropertyId::GridColumnEnd,
                    GridLineValue::Named("main".into())
                ),
            ]
        );
    }

    #[test]
    fn grid_column_slash_form_expands_to_two_line_longhands() {
        // `grid-column: 1 / 3` -> start = line 1, end = line 3.
        assert_eq!(
            grid_lines(&compile_all("grid-column: 1 / 3")),
            vec![
                (PropertyId::GridColumnStart, GridLineValue::Line(1)),
                (PropertyId::GridColumnEnd, GridLineValue::Line(3)),
            ]
        );
    }

    #[test]
    fn grid_column_single_line_sets_end_auto() {
        // `grid-column: 2` -> start = line 2, end = auto.
        assert_eq!(
            grid_lines(&compile_all("grid-column: 2")),
            vec![
                (PropertyId::GridColumnStart, GridLineValue::Line(2)),
                (PropertyId::GridColumnEnd, GridLineValue::Auto),
            ]
        );
    }

    #[test]
    fn grid_column_span_form_sets_start_span_end_auto() {
        // CSS-faithful: a single non-ident `grid-column: span 2` sets the *start*
        // edge to `span 2` and the end edge to `auto` (the item spans 2 columns,
        // auto-placed).
        assert_eq!(
            grid_lines(&compile_all("grid-column: span 2")),
            vec![
                (PropertyId::GridColumnStart, GridLineValue::Span(2)),
                (PropertyId::GridColumnEnd, GridLineValue::Auto),
            ]
        );
    }

    #[test]
    fn grid_row_single_line_expands() {
        // `grid-row: 2` -> start = line 2, end = auto.
        assert_eq!(
            grid_lines(&compile_all("grid-row: 2")),
            vec![
                (PropertyId::GridRowStart, GridLineValue::Line(2)),
                (PropertyId::GridRowEnd, GridLineValue::Auto),
            ]
        );
    }

    #[test]
    fn grid_row_span_slash_line_end_anchored() {
        // `grid-row: span 2 / 5` -> start = span 2, end = line 5.
        assert_eq!(
            grid_lines(&compile_all("grid-row: span 2 / 5")),
            vec![
                (PropertyId::GridRowStart, GridLineValue::Span(2)),
                (PropertyId::GridRowEnd, GridLineValue::Line(5)),
            ]
        );
    }

    #[test]
    fn grid_area_four_values_expand_in_css_order() {
        // `grid-area: <row-start> / <col-start> / <row-end> / <col-end>`.
        assert_eq!(
            grid_lines(&compile_all("grid-area: 1 / 2 / 3 / 4")),
            vec![
                (PropertyId::GridRowStart, GridLineValue::Line(1)),
                (PropertyId::GridColumnStart, GridLineValue::Line(2)),
                (PropertyId::GridRowEnd, GridLineValue::Line(3)),
                (PropertyId::GridColumnEnd, GridLineValue::Line(4)),
            ]
        );
    }

    #[test]
    fn grid_area_mixes_line_and_span_edges() {
        // `grid-area: 1 / 1 / span 2 / span 2`.
        assert_eq!(
            grid_lines(&compile_all("grid-area: 1 / 1 / span 2 / span 2")),
            vec![
                (PropertyId::GridRowStart, GridLineValue::Line(1)),
                (PropertyId::GridColumnStart, GridLineValue::Line(1)),
                (PropertyId::GridRowEnd, GridLineValue::Span(2)),
                (PropertyId::GridColumnEnd, GridLineValue::Span(2)),
            ]
        );
    }

    #[test]
    fn grid_area_two_values_leave_ends_auto() {
        // `grid-area: 2 / 3` → row-start=2, col-start=3, both ends auto.
        assert_eq!(
            grid_lines(&compile_all("grid-area: 2 / 3")),
            vec![
                (PropertyId::GridRowStart, GridLineValue::Line(2)),
                (PropertyId::GridColumnStart, GridLineValue::Line(3)),
                (PropertyId::GridRowEnd, GridLineValue::Auto),
                (PropertyId::GridColumnEnd, GridLineValue::Auto),
            ]
        );
    }

    #[test]
    fn grid_area_single_number_leaves_other_edges_auto() {
        // A number only sets the first edge.
        assert_eq!(
            grid_lines(&compile_all("grid-area: 2")),
            vec![
                (PropertyId::GridRowStart, GridLineValue::Line(2)),
                (PropertyId::GridColumnStart, GridLineValue::Auto),
                (PropertyId::GridRowEnd, GridLineValue::Auto),
                (PropertyId::GridColumnEnd, GridLineValue::Auto),
            ]
        );
    }

    #[test]
    fn grid_area_negative_end_lines_carry_through() {
        // `grid-area: 1 / 1 / -1 / -1` (a full-grid cell) keeps the negative lines
        // for the layout to resolve against the track count.
        assert_eq!(
            grid_lines(&compile_all("grid-area: 1 / 1 / -1 / -1")),
            vec![
                (PropertyId::GridRowStart, GridLineValue::Line(1)),
                (PropertyId::GridColumnStart, GridLineValue::Line(1)),
                (PropertyId::GridRowEnd, GridLineValue::Line(-1)),
                (PropertyId::GridColumnEnd, GridLineValue::Line(-1)),
            ]
        );
    }

    #[test]
    fn grid_area_named_ident_sets_all_four_edges() {
        assert_eq!(
            grid_lines(&compile_all("grid-area: header")),
            [
                PropertyId::GridRowStart,
                PropertyId::GridColumnStart,
                PropertyId::GridRowEnd,
                PropertyId::GridColumnEnd,
            ]
            .map(|property| (property, GridLineValue::Named("header".into())))
        );
    }

    #[test]
    fn grid_area_copies_each_bare_start_name_to_its_omitted_end() {
        assert_eq!(
            grid_lines(&compile_all("grid-area: section / 2")),
            [
                (
                    PropertyId::GridRowStart,
                    GridLineValue::Named("section".into())
                ),
                (PropertyId::GridColumnStart, GridLineValue::Line(2)),
                (
                    PropertyId::GridRowEnd,
                    GridLineValue::Named("section".into())
                ),
                (PropertyId::GridColumnEnd, GridLineValue::Auto),
            ]
        );
    }

    #[test]
    fn grid_area_more_than_four_values_is_unsupported() {
        // More than four line values is invalid.
        let decls = compile_all("grid-area: 1 / 2 / 3 / 4 / 5");
        assert_eq!(decls.len(), 1);
        assert!(matches!(decls[0], CompiledDeclaration::Unsupported(_)));
    }

    /// `(grow, shrink, basis)` for a `flex` shorthand, as the three longhands.
    fn flex_triple(text: &str) -> Vec<(PropertyId, PropertyValue)> {
        pairs(&compile_all(text))
    }

    fn flex_expected(
        grow: f32,
        shrink: f32,
        basis: PropertyValue,
    ) -> Vec<(PropertyId, PropertyValue)> {
        vec![
            (PropertyId::FlexGrow, PropertyValue::Number(grow)),
            (PropertyId::FlexShrink, PropertyValue::Number(shrink)),
            (PropertyId::FlexBasis, basis),
        ]
    }

    #[test]
    fn flex_shorthand_three_value_form_maps_positionally() {
        let basis = PropertyValue::Length(Length::px(100.0));
        assert_eq!(
            flex_triple("flex: 2 3 100px"),
            flex_expected(2.0, 3.0, basis)
        );
        assert_eq!(
            flex_triple("flex: 0 1 auto"),
            flex_expected(0.0, 1.0, value_of("flex-basis: auto"))
        );
        // A unitless zero preceded by two flex factors is a `0px` basis.
        assert_eq!(
            flex_triple("flex: 1 1 0"),
            flex_expected(1.0, 1.0, PropertyValue::Length(Length::px(0.0)))
        );
    }

    #[test]
    fn flex_shorthand_omitted_components_default_to_one_one_zero() {
        let zero = PropertyValue::Length(Length::px(0.0));
        // A lone factor: `flex: 1` is `1 1 0px`, NOT `1 1 auto` (§7.1 note).
        assert_eq!(
            flex_triple("flex: 1"),
            flex_expected(1.0, 1.0, zero.clone())
        );
        assert_eq!(
            flex_triple("flex: 0"),
            flex_expected(0.0, 1.0, zero.clone())
        );
        // Two factors: the basis still defaults to `0px`.
        assert_eq!(flex_triple("flex: 2 0"), flex_expected(2.0, 0.0, zero));
        // A lone basis: both factors default to `1` (`flex: auto` is `1 1 auto`).
        assert_eq!(
            flex_triple("flex: auto"),
            flex_expected(1.0, 1.0, value_of("flex-basis: auto"))
        );
        assert_eq!(
            flex_triple("flex: 30%"),
            flex_expected(1.0, 1.0, value_of("flex-basis: 30%"))
        );
        // `<grow> <basis>` and `<basis> <grow>` are both valid (`||` is unordered).
        let hundred = PropertyValue::Length(Length::px(100.0));
        assert_eq!(
            flex_triple("flex: 2 100px"),
            flex_expected(2.0, 1.0, hundred.clone())
        );
        assert_eq!(
            flex_triple("flex: 100px 2"),
            flex_expected(2.0, 1.0, hundred)
        );
    }

    #[test]
    fn flex_none_and_css_wide_keywords() {
        // `none` expands to `0 0 auto`.
        assert_eq!(
            flex_triple("flex: none"),
            flex_expected(0.0, 0.0, value_of("flex-basis: auto"))
        );
        // A CSS-wide keyword reaches every longhand unchanged; `initial` therefore
        // lands on the longhand initial values (`0 1 auto`), not on `1 1 0px`.
        assert_eq!(
            flex_triple("flex: initial"),
            vec![
                (PropertyId::FlexGrow, PropertyValue::Initial),
                (PropertyId::FlexShrink, PropertyValue::Initial),
                (PropertyId::FlexBasis, PropertyValue::Initial),
            ]
        );
        assert_eq!(
            flex_triple("flex: inherit"),
            vec![
                (PropertyId::FlexGrow, PropertyValue::Inherit),
                (PropertyId::FlexShrink, PropertyValue::Inherit),
                (PropertyId::FlexBasis, PropertyValue::Inherit),
            ]
        );
    }

    #[test]
    fn invalid_flex_shorthand_degrades_to_unsupported() {
        for text in [
            // A non-zero unitless number cannot be a `<'flex-basis'>`.
            "flex: 1 1 2",
            // Negative flex factors are outside `<number [0,∞]>`.
            "flex: -1 1 0px",
            // Two basis components, no factor.
            "flex: auto 100px",
            // More components than the grammar allows.
            "flex: 1 1 1 1px",
        ] {
            assert!(
                matches!(
                    compile_all(text).as_slice(),
                    [CompiledDeclaration::Unsupported(_)]
                ),
                "expected `{text}` to degrade to unsupported"
            );
        }
    }
}
