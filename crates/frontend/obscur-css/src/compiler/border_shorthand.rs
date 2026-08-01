#![doc = include_str!("border_shorthand.doc/_module.md")]

use super::color_lowering::{color_from_function, named_color, parse_hex_color};
use super::property_compiler::{CompiledDeclaration, value_unsupported_kind};
use super::stylesheet_compiler::CssSymbolInterner;
use super::value_lowering::dimension_length;
use crate::error::UnsupportedCssFeature;
use crate::parser::{ParsedDeclaration, ParsedValue};
use obscur_style::cascade::{CascadeOrigin, Declaration, ImportantFlag};
use obscur_style::property::{Length, PropertyId, PropertyValue, layout_keyword};
use obscur_style::selector::{SelectorId, SelectorSpecificity};

#[doc = include_str!("border_shorthand.doc/BorderSide.md")]
pub(super) struct BorderSide {
    #[doc = include_str!("border_shorthand.doc/width.md")]
    width: PropertyId,
    #[doc = include_str!("border_shorthand.doc/style.md")]
    style: PropertyId,
    #[doc = include_str!("border_shorthand.doc/color.md")]
    color: PropertyId,
}

#[doc = include_str!("border_shorthand.doc/BORDER_SIDES.md")]
pub(super) const BORDER_SIDES: &[BorderSide] = &[
    BorderSide {
        width: PropertyId::BorderTopWidth,
        style: PropertyId::BorderTopStyle,
        color: PropertyId::BorderTopColor,
    },
    BorderSide {
        width: PropertyId::BorderRightWidth,
        style: PropertyId::BorderRightStyle,
        color: PropertyId::BorderRightColor,
    },
    BorderSide {
        width: PropertyId::BorderBottomWidth,
        style: PropertyId::BorderBottomStyle,
        color: PropertyId::BorderBottomColor,
    },
    BorderSide {
        width: PropertyId::BorderLeftWidth,
        style: PropertyId::BorderLeftStyle,
        color: PropertyId::BorderLeftColor,
    },
];

#[doc = include_str!("border_shorthand.doc/border_side_shorthand.md")]
pub(super) fn border_side_shorthand(name: &str) -> Option<&'static [BorderSide]> {
    if name.eq_ignore_ascii_case("border-top") {
        Some(&BORDER_SIDES[0..1])
    } else if name.eq_ignore_ascii_case("border-right") {
        Some(&BORDER_SIDES[1..2])
    } else if name.eq_ignore_ascii_case("border-bottom") {
        Some(&BORDER_SIDES[2..3])
    } else if name.eq_ignore_ascii_case("border-left") {
        Some(&BORDER_SIDES[3..4])
    } else {
        None
    }
}

#[doc = include_str!("border_shorthand.doc/BorderComponents.md")]
struct BorderComponents {
    width: Option<PropertyValue>,
    style: Option<PropertyValue>,
    color: Option<PropertyValue>,
}

#[doc = include_str!("border_shorthand.doc/classify_border.md")]
fn classify_border(
    values: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<BorderComponents> {
    let mut out = BorderComponents {
        width: None,
        style: None,
        color: None,
    };
    for value in values {
        // A `<line-style>` keyword (`solid` / `none` / …) takes priority over the
        // color reading of the same keyword token.
        if let ParsedValue::Keyword(keyword) = value
            && let Some(sym) = layout_keyword::border_style_from_name(keyword)
        {
            if out.style.is_some() {
                return None;
            }
            out.style = Some(PropertyValue::Keyword(sym));
            continue;
        }
        if let Some(width) = border_width_component(value) {
            if out.width.is_some() {
                return None;
            }
            out.width = Some(width);
            continue;
        }
        if let Some(color) = border_color_component(value, interner) {
            if out.color.is_some() {
                return None;
            }
            out.color = Some(color);
            continue;
        }
        return None;
    }
    Some(out)
}

#[doc = include_str!("border_shorthand.doc/border_width_component.md")]
fn border_width_component(value: &ParsedValue) -> Option<PropertyValue> {
    match value {
        ParsedValue::Number(n) if *n == 0.0 => Some(PropertyValue::Length(Length::px(0.0))),
        ParsedValue::Dimension { value, unit } => {
            dimension_length(*value, unit).map(PropertyValue::Length)
        }
        _ => None,
    }
}

#[doc = include_str!("border_shorthand.doc/border_color_component.md")]
fn border_color_component(
    value: &ParsedValue,
    _interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    match value {
        ParsedValue::Hash(hex) => parse_hex_color(hex).map(PropertyValue::Color),
        ParsedValue::Keyword(keyword) => named_color(keyword).map(PropertyValue::Color),
        ParsedValue::Function { name, args } => {
            color_from_function(name, args).map(PropertyValue::Color)
        }
        _ => None,
    }
}

#[doc = include_str!("border_shorthand.doc/expand_border.md")]
#[allow(clippy::too_many_arguments)]
pub(super) fn expand_border(
    decl: &ParsedDeclaration,
    sides: &[BorderSide],
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
    let Some(components) = classify_border(&decl.value, interner) else {
        return unsupported();
    };
    // An empty value (no width / style / color) authors nothing to expand.
    if components.width.is_none() && components.style.is_none() && components.color.is_none() {
        return unsupported();
    }
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    let mut out = Vec::new();
    for side in sides {
        for (property, value) in [
            (side.width, &components.width),
            (side.style, &components.style),
            (side.color, &components.color),
        ] {
            if let Some(value) = value {
                out.push(CompiledDeclaration::Standard(Declaration::new(
                    property,
                    value.clone(),
                    important,
                    origin,
                    specificity,
                    source_order,
                    selector,
                )));
            }
        }
    }
    out
}

#[doc = include_str!("border_shorthand.doc/expand_outline.md")]
pub(super) fn expand_outline(
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
    let Some(components) = classify_border(&decl.value, interner) else {
        return unsupported();
    };
    if components.width.is_none() && components.style.is_none() && components.color.is_none() {
        return unsupported();
    }
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    let mut out = Vec::new();
    for (property, value) in [
        (PropertyId::OutlineWidth, &components.width),
        (PropertyId::OutlineStyle, &components.style),
        (PropertyId::OutlineColor, &components.color),
    ] {
        if let Some(value) = value {
            out.push(CompiledDeclaration::Standard(Declaration::new(
                property,
                value.clone(),
                important,
                origin,
                specificity,
                source_order,
                selector,
            )));
        }
    }
    out
}

#[doc = include_str!("border_shorthand.doc/expand_border_color.md")]
pub(super) fn expand_border_color(
    decl: &ParsedDeclaration,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
    interner: &mut CssSymbolInterner,
) -> Vec<CompiledDeclaration> {
    let ids = [
        PropertyId::BorderTopColor,
        PropertyId::BorderRightColor,
        PropertyId::BorderBottomColor,
        PropertyId::BorderLeftColor,
    ];
    let mut components = Vec::new();
    for value in &decl.value {
        match border_color_component(value, interner) {
            Some(color) => components.push(color),
            None => return border_side_unsupported(decl),
        }
    }
    emit_four(
        decl,
        ids,
        &components,
        specificity,
        origin,
        source_order,
        selector,
    )
}

#[doc = include_str!("border_shorthand.doc/expand_border_style.md")]
pub(super) fn expand_border_style(
    decl: &ParsedDeclaration,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
) -> Vec<CompiledDeclaration> {
    let ids = [
        PropertyId::BorderTopStyle,
        PropertyId::BorderRightStyle,
        PropertyId::BorderBottomStyle,
        PropertyId::BorderLeftStyle,
    ];
    let mut components = Vec::new();
    for value in &decl.value {
        match value {
            ParsedValue::Keyword(keyword) => {
                match layout_keyword::border_style_from_name(keyword) {
                    Some(sym) => components.push(PropertyValue::Keyword(sym)),
                    None => return border_side_unsupported(decl),
                }
            }
            _ => return border_side_unsupported(decl),
        }
    }
    emit_four(
        decl,
        ids,
        &components,
        specificity,
        origin,
        source_order,
        selector,
    )
}

#[doc = include_str!("border_shorthand.doc/border_side_unsupported.md")]
fn border_side_unsupported(decl: &ParsedDeclaration) -> Vec<CompiledDeclaration> {
    vec![CompiledDeclaration::Unsupported(
        UnsupportedCssFeature::new(value_unsupported_kind(&decl.value), &*decl.name, decl.span),
    )]
}

#[doc = include_str!("border_shorthand.doc/emit_four.md")]
fn emit_four(
    decl: &ParsedDeclaration,
    ids: [PropertyId; 4],
    components: &[PropertyValue],
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
) -> Vec<CompiledDeclaration> {
    let four = match components {
        [a] => [a.clone(), a.clone(), a.clone(), a.clone()],
        [a, b] => [a.clone(), b.clone(), a.clone(), b.clone()],
        [a, b, c] => [a.clone(), b.clone(), c.clone(), b.clone()],
        [a, b, c, d] => [a.clone(), b.clone(), c.clone(), d.clone()],
        _ => return border_side_unsupported(decl),
    };
    let important = if decl.important {
        ImportantFlag::Important
    } else {
        ImportantFlag::Normal
    };
    ids.into_iter()
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

#[cfg(test)]
mod tests {
    use super::super::longhand_catalog::property_id_from_name;
    use super::super::property_compiler::tests::{compile_all, pairs, value_of};

    use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor, layout_keyword};

    #[test]
    fn border_shorthand_sets_width_style_color_on_all_four_sides() {
        let compiled = compile_all("border: 2px solid #ff0000");
        let got = pairs(&compiled);
        let width = PropertyValue::Length(Length::px(2.0));
        let style = PropertyValue::Keyword(layout_keyword::BORDER_STYLE_SOLID);
        let color = PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(
            got,
            vec![
                (PropertyId::BorderTopWidth, width.clone()),
                (PropertyId::BorderTopStyle, style.clone()),
                (PropertyId::BorderTopColor, color.clone()),
                (PropertyId::BorderRightWidth, width.clone()),
                (PropertyId::BorderRightStyle, style.clone()),
                (PropertyId::BorderRightColor, color.clone()),
                (PropertyId::BorderBottomWidth, width.clone()),
                (PropertyId::BorderBottomStyle, style.clone()),
                (PropertyId::BorderBottomColor, color.clone()),
                (PropertyId::BorderLeftWidth, width),
                (PropertyId::BorderLeftStyle, style),
                (PropertyId::BorderLeftColor, color),
            ]
        );
    }

    #[test]
    fn border_shorthand_accepts_components_in_any_order() {
        // `red solid 2px` classifies identically to `2px solid red`.
        let got = pairs(&compile_all("border: red solid 2px"));
        assert!(got.contains(&(
            PropertyId::BorderTopWidth,
            PropertyValue::Length(Length::px(2.0))
        )));
        assert!(got.contains(&(
            PropertyId::BorderTopStyle,
            PropertyValue::Keyword(layout_keyword::BORDER_STYLE_SOLID)
        )));
        assert!(got.contains(&(
            PropertyId::BorderTopColor,
            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        )));
    }

    #[test]
    fn border_side_shorthand_sets_one_edge() {
        let got = pairs(&compile_all("border-top: 1px solid red"));
        assert_eq!(
            got,
            vec![
                (
                    PropertyId::BorderTopWidth,
                    PropertyValue::Length(Length::px(1.0))
                ),
                (
                    PropertyId::BorderTopStyle,
                    PropertyValue::Keyword(layout_keyword::BORDER_STYLE_SOLID)
                ),
                (
                    PropertyId::BorderTopColor,
                    PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
                ),
            ]
        );
    }

    #[test]
    fn border_color_two_value_replicates_to_four_sides() {
        // `red green` → top/bottom = red, right/left = green (CSS 2-value rule).
        let red = PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
        let green = PropertyValue::Color(StyleColor::rgba(0.0, 128.0 / 255.0, 0.0, 1.0));
        assert_eq!(
            pairs(&compile_all("border-color: red green")),
            vec![
                (PropertyId::BorderTopColor, red.clone()),
                (PropertyId::BorderRightColor, green.clone()),
                (PropertyId::BorderBottomColor, red),
                (PropertyId::BorderLeftColor, green),
            ]
        );
    }

    #[test]
    fn outline_shorthand_sets_width_style_color() {
        // `2px dashed #333` → the three outline longhands, dashed style handle.
        let got = pairs(&compile_all("outline: 2px dashed #333333"));
        assert_eq!(
            got,
            vec![
                (
                    PropertyId::OutlineWidth,
                    PropertyValue::Length(Length::px(2.0))
                ),
                (
                    PropertyId::OutlineStyle,
                    PropertyValue::Keyword(layout_keyword::BORDER_STYLE_DASHED)
                ),
                (
                    PropertyId::OutlineColor,
                    PropertyValue::Color(StyleColor::rgba(0.2, 0.2, 0.2, 1.0))
                ),
            ]
        );
    }

    #[test]
    fn outline_shorthand_accepts_components_in_any_order() {
        // `#333 solid 2px` classifies identically to `2px solid #333`.
        let got = pairs(&compile_all("outline: #333333 solid 2px"));
        assert!(got.contains(&(
            PropertyId::OutlineWidth,
            PropertyValue::Length(Length::px(2.0))
        )));
        assert!(got.contains(&(
            PropertyId::OutlineStyle,
            PropertyValue::Keyword(layout_keyword::BORDER_STYLE_SOLID)
        )));
        assert!(got.contains(&(
            PropertyId::OutlineColor,
            PropertyValue::Color(StyleColor::rgba(0.2, 0.2, 0.2, 1.0))
        )));
    }

    #[test]
    fn outline_offset_lowers_to_a_length() {
        assert_eq!(
            pairs(&compile_all("outline-offset: 3px")),
            vec![(
                PropertyId::OutlineOffset,
                PropertyValue::Length(Length::px(3.0))
            )]
        );
    }

    #[test]
    fn outline_style_none_lowers_to_the_none_handle() {
        assert_eq!(
            pairs(&compile_all("outline-style: none")),
            vec![(
                PropertyId::OutlineStyle,
                PropertyValue::Keyword(layout_keyword::BORDER_STYLE_NONE)
            )]
        );
    }

    #[test]
    fn outline_none_shorthand_lowers_to_the_none_style() {
        // `outline: none` classifies its lone keyword as the style (like `border`).
        assert_eq!(
            pairs(&compile_all("outline: none")),
            vec![(
                PropertyId::OutlineStyle,
                PropertyValue::Keyword(layout_keyword::BORDER_STYLE_NONE)
            )]
        );
    }

    #[test]
    fn border_width_and_style_shorthands_fan_out() {
        assert_eq!(
            pairs(&compile_all("border-width: 3px")),
            vec![
                (
                    PropertyId::BorderTopWidth,
                    PropertyValue::Length(Length::px(3.0))
                ),
                (
                    PropertyId::BorderRightWidth,
                    PropertyValue::Length(Length::px(3.0))
                ),
                (
                    PropertyId::BorderBottomWidth,
                    PropertyValue::Length(Length::px(3.0))
                ),
                (
                    PropertyId::BorderLeftWidth,
                    PropertyValue::Length(Length::px(3.0))
                ),
            ]
        );
        let solid = PropertyValue::Keyword(layout_keyword::BORDER_STYLE_SOLID);
        assert_eq!(
            pairs(&compile_all("border-style: solid")),
            vec![
                (PropertyId::BorderTopStyle, solid.clone()),
                (PropertyId::BorderRightStyle, solid.clone()),
                (PropertyId::BorderBottomStyle, solid.clone()),
                (PropertyId::BorderLeftStyle, solid),
            ]
        );
    }

    #[test]
    fn border_longhands_resolve_directly() {
        assert_eq!(
            property_id_from_name("border-top-width"),
            Some(PropertyId::BorderTopWidth)
        );
        assert_eq!(
            value_of("border-left-color: #00ff00"),
            PropertyValue::Color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0))
        );
        assert_eq!(
            value_of("border-right-style: dashed"),
            PropertyValue::Keyword(layout_keyword::BORDER_STYLE_DASHED)
        );
    }
}
