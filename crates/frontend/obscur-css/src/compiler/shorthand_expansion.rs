#![doc = include_str!("shorthand_expansion.doc/_module.md")]

use super::grid_placement::grid_line_edge;
use super::property_compiler::{CompiledDeclaration, lower_for_property, value_unsupported_kind};
use super::stylesheet_compiler::CssSymbolInterner;
use super::value_lowering::{css_wide_keyword, lower_component};
use crate::error::UnsupportedCssFeature;
use crate::parser::{ParsedDeclaration, ParsedValue};
use obscur_style::cascade::{CascadeOrigin, Declaration, ImportantFlag};
use obscur_style::property::{GridLineValue, Length, PropertyId, PropertyValue};
use obscur_style::selector::{SelectorId, SelectorSpecificity};

#[doc = include_str!("shorthand_expansion.doc/box_shorthand_sides.md")]
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

#[doc = include_str!("shorthand_expansion.doc/expand_box_shorthand.md")]
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
    let mut components = Vec::new();
    for value in &decl.value {
        match box_side_length(value, interner) {
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

#[doc = include_str!("shorthand_expansion.doc/place_shorthand_sides.md")]
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

#[doc = include_str!("shorthand_expansion.doc/logical_pair_shorthand_sides.md")]
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

#[doc = include_str!("shorthand_expansion.doc/expand_pair_shorthand.md")]
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
    let (align_values, justify_values): (&[ParsedValue], &[ParsedValue]) = match &*decl.value {
        [a] => (std::slice::from_ref(a), std::slice::from_ref(a)),
        [a, b] => (std::slice::from_ref(a), std::slice::from_ref(b)),
        _ => return unsupported(),
    };
    let (Some(align_value), Some(justify_value)) = (
        lower_for_property(align, align_values, interner),
        lower_for_property(justify, justify_values, interner),
    ) else {
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

#[doc = include_str!("shorthand_expansion.doc/expand_flex_shorthand.md")]
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

#[doc = include_str!("shorthand_expansion.doc/grid_line_shorthand_sides.md")]
pub(super) fn grid_line_shorthand_sides(name: &str) -> Option<(PropertyId, PropertyId)> {
    if name.eq_ignore_ascii_case("grid-column") {
        Some((PropertyId::GridColumnStart, PropertyId::GridColumnEnd))
    } else if name.eq_ignore_ascii_case("grid-row") {
        Some((PropertyId::GridRowStart, PropertyId::GridRowEnd))
    } else {
        None
    }
}

#[doc = include_str!("shorthand_expansion.doc/expand_grid_line_shorthand.md")]
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
    // No `/`: the end edge is `auto` (the line-based single-value form).
    let end = match end_group {
        Some(group) => match grid_line_edge(group) {
            Some(edge) => edge,
            None => return unsupported(),
        },
        None => GridLineValue::Auto,
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

#[doc = include_str!("shorthand_expansion.doc/expand_grid_area_shorthand.md")]
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
    // The single-`<custom-ident>` named form (`grid-area: header`): a lone keyword
    // that is not a CSS-wide keyword and not the `auto` line edge is an area name.
    // It cannot be resolved to line numbers here (the area rectangle lives on the
    // item's *container* `grid-template-areas`), so it is carried as a
    // `PropertyId::GridArea` marker the runtime resolves against the parent's
    // template. The numeric `a / b / c / d` line form falls through below.
    if let [ParsedValue::Keyword(name)] = decl.value.as_slice()
        && css_wide_keyword(name).is_none()
        && !name.eq_ignore_ascii_case("auto")
    {
        return vec![CompiledDeclaration::Standard(Declaration::new(
            PropertyId::GridArea,
            PropertyValue::GridAreaName((**name).into()),
            important,
            origin,
            specificity,
            source_order,
            selector,
        ))];
    }
    let mut edges = Vec::new();
    for group in decl
        .value
        .split(|v| matches!(v, ParsedValue::Delimiter('/')))
    {
        match grid_line_edge(group) {
            Some(edge) => edges.push(edge),
            None => return unsupported(),
        }
    }
    // `GridLineValue` is no longer `Copy` (its `Named` variant owns a name), so
    // clone the reused edges.
    let [row_start, column_start, row_end, column_end] = match edges.as_slice() {
        [a] => [a.clone(), a.clone(), a.clone(), a.clone()],
        [a, b] => [
            a.clone(),
            b.clone(),
            GridLineValue::Auto,
            GridLineValue::Auto,
        ],
        [a, b, c] => [a.clone(), b.clone(), c.clone(), GridLineValue::Auto],
        [a, b, c, d] => [a.clone(), b.clone(), c.clone(), d.clone()],
        _ => return unsupported(),
    };
    [
        (PropertyId::GridRowStart, row_start),
        (PropertyId::GridColumnStart, column_start),
        (PropertyId::GridRowEnd, row_end),
        (PropertyId::GridColumnEnd, column_end),
    ]
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

#[doc = include_str!("shorthand_expansion.doc/box_side_length.md")]
fn box_side_length(value: &ParsedValue, interner: &mut CssSymbolInterner) -> Option<PropertyValue> {
    if let ParsedValue::Number(n) = value
        && *n == 0.0
    {
        return Some(PropertyValue::Length(Length::px(0.0)));
    }
    match lower_component(value, interner) {
        // A math function is accepted wherever a `<length>` is (CSS Values 4
        // §10.1), and these shorthands are no exception: rejecting it here does
        // not degrade one corner, it degrades the WHOLE declaration to
        // `Unsupported`, so `border-radius: min(3vmin, 14px)` used to leave four
        // square corners rather than four clamped ones.
        Some(length @ (PropertyValue::Length(_) | PropertyValue::Calc(_))) => Some(length),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::grid_placement::tests::grid_lines;
    use super::super::longhand_catalog::property_id_from_name;
    use super::super::property_compiler::tests::{compile_all, pairs, standard_of, value_of};

    use crate::compiler::CompiledDeclaration;

    use obscur_style::property::{
        GridLineValue, Length, PropertyId, PropertyValue, layout_keyword,
    };

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
    fn grid_column_single_named_line_sets_end_auto() {
        // `grid-column: main` -> start = Named("main"), end = auto.
        assert_eq!(
            grid_lines(&compile_all("grid-column: main")),
            vec![
                (
                    PropertyId::GridColumnStart,
                    GridLineValue::Named("main".into())
                ),
                (PropertyId::GridColumnEnd, GridLineValue::Auto),
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
    fn grid_area_single_value_fills_all_four() {
        // `grid-area: 2` → all four edges = line 2.
        assert_eq!(
            grid_lines(&compile_all("grid-area: 2")),
            vec![
                (PropertyId::GridRowStart, GridLineValue::Line(2)),
                (PropertyId::GridColumnStart, GridLineValue::Line(2)),
                (PropertyId::GridRowEnd, GridLineValue::Line(2)),
                (PropertyId::GridColumnEnd, GridLineValue::Line(2)),
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
    fn grid_area_named_ident_lowers_to_marker() {
        // A bare named-area ident is a `GridArea` marker the runtime resolves.
        let decls = compile_all("grid-area: header");
        assert_eq!(decls.len(), 1);
        let CompiledDeclaration::Standard(d) = &decls[0] else {
            panic!("expected standard, got {:?}", decls[0]);
        };
        assert_eq!(d.property(), PropertyId::GridArea);
        assert_eq!(d.value(), &PropertyValue::GridAreaName("header".into()));
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
