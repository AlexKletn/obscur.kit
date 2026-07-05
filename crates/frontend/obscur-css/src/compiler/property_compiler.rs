//! Lowering a single parsed declaration into an `obscur-style` value (§4.14).
//!
//! [`PropertyCompiler`] maps one [`ParsedDeclaration`] onto exactly one
//! [`CompiledDeclaration`]:
//!
//! * a name matching one of the seven [`PropertyId`] longhands becomes a
//!   cascade [`Declaration`] ([`CompiledDeclaration::Standard`]);
//! * a `--`-prefixed name becomes a [`CustomProperty`]
//!   ([`CompiledDeclaration::Custom`]);
//! * any other recognised-but-unsupported property name, or a value the crate
//!   cannot lower, becomes an [`UnsupportedCssFeature`]
//!   ([`CompiledDeclaration::Unsupported`]) rather than being dropped silently
//!   (§4.13).
//!
//! Value lowering handles the single-component cases the domain models today
//! (colours, lengths, numbers, keywords, `var()` references and the CSS-wide
//! keywords). Multi-component shorthands are beyond the domain's current value
//! model, so lowering is best-effort on the first component.

use crate::error::{UnsupportedCssFeature, UnsupportedFeatureKind};
use crate::parser::{ParsedDeclaration, ParsedValue};

use obscur_identity::InternedSymbol;
use obscur_style::cascade::{CascadeOrigin, Declaration, ImportantFlag};
use obscur_style::property::{
    layout_keyword, CustomProperty, GridAreaTemplate, GridLineValue, GridTrack, GridTrackList,
    Length, PropertyId, PropertyValue, StyleColor,
};
use obscur_style::selector::{SelectorId, SelectorSpecificity};

use super::calc_lowering::lower_math_function;
use super::stylesheet_compiler::CssSymbolInterner;

/// The result of lowering one [`ParsedDeclaration`].
///
/// A declaration lowers to exactly one of these: a standard cascade
/// [`Declaration`], a [`CustomProperty`], or an [`UnsupportedCssFeature`]
/// diagnostic when the property name or its value cannot be lowered.
#[derive(Debug, Clone, PartialEq)]
pub enum CompiledDeclaration {
    /// A standard longhand declaration ready for the cascade.
    Standard(Declaration),
    /// A `--`-prefixed custom property.
    Custom(CustomProperty),
    /// A recognised-but-unsupported property or value (§4.13).
    Unsupported(UnsupportedCssFeature),
}

/// Lowers one parsed declaration into a [`CompiledDeclaration`].
///
/// The compiler is stateless; the string interner it needs is threaded in per
/// call so a single instance may lower any number of declarations.
#[derive(Debug, Clone, Copy, Default)]
pub struct PropertyCompiler;

impl PropertyCompiler {
    /// Creates a property compiler.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Lowers `decl` into a [`CompiledDeclaration`], resolving the cascade
    /// context (`specificity`, `origin`, `source_order`, `selector`) into the
    /// produced [`Declaration`] when the name is a known longhand.
    #[must_use]
    pub fn compile(
        &self,
        decl: &ParsedDeclaration,
        specificity: SelectorSpecificity,
        origin: CascadeOrigin,
        source_order: u32,
        selector: Option<SelectorId>,
        interner: &mut CssSymbolInterner,
    ) -> CompiledDeclaration {
        let name = &*decl.name;
        if name.starts_with("--") {
            // A custom property keeps its value verbatim (best-effort); an
            // unlowerable value degrades to `initial` rather than being dropped.
            let value = lower_value(&decl.value, interner).unwrap_or(PropertyValue::Initial);
            let symbol = interner.intern(name);
            return CompiledDeclaration::Custom(CustomProperty::new(symbol, value));
        }
        let Some(property) = property_id_from_name(name) else {
            return CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
                UnsupportedFeatureKind::Property,
                name,
                decl.span,
            ));
        };
        match lower_for_property(property, &decl.value, interner) {
            Some(value) => {
                let important = if decl.important {
                    ImportantFlag::Important
                } else {
                    ImportantFlag::Normal
                };
                CompiledDeclaration::Standard(Declaration::new(
                    property,
                    value,
                    important,
                    origin,
                    specificity,
                    source_order,
                    selector,
                ))
            }
            None => CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
                value_unsupported_kind(&decl.value),
                name,
                decl.span,
            )),
        }
    }

    /// Lowers `decl` into one or more [`CompiledDeclaration`]s, expanding the
    /// `margin` / `padding` box shorthands into their four longhands.
    ///
    /// Every non-shorthand declaration lowers to exactly one result (the single
    /// [`compile`](Self::compile) call, wrapped in a one-element `Vec`). A
    /// `margin` / `padding` shorthand expands into up to four
    /// [`CompiledDeclaration::Standard`] longhands (the CSS 1/2/3/4-value forms);
    /// an unparseable shorthand value yields a single
    /// [`CompiledDeclaration::Unsupported`]. This is the entry point the
    /// stylesheet compiler uses so shorthands flow into both the flat cascade and
    /// the matchable block.
    #[must_use]
    pub fn compile_all(
        &self,
        decl: &ParsedDeclaration,
        specificity: SelectorSpecificity,
        origin: CascadeOrigin,
        source_order: u32,
        selector: Option<SelectorId>,
        interner: &mut CssSymbolInterner,
    ) -> Vec<CompiledDeclaration> {
        if let Some(sides) = box_shorthand_sides(&decl.name) {
            return expand_box_shorthand(
                decl,
                sides,
                specificity,
                origin,
                source_order,
                selector,
                interner,
            );
        }
        if decl.name.eq_ignore_ascii_case("border-color") {
            return expand_border_color(decl, specificity, origin, source_order, selector, interner);
        }
        if decl.name.eq_ignore_ascii_case("border-style") {
            return expand_border_style(decl, specificity, origin, source_order, selector);
        }
        if decl.name.eq_ignore_ascii_case("border") {
            return expand_border(decl, BORDER_SIDES, specificity, origin, source_order, selector, interner);
        }
        if let Some(side) = border_side_shorthand(&decl.name) {
            return expand_border(decl, side, specificity, origin, source_order, selector, interner);
        }
        if let Some((start, end)) = grid_line_shorthand_sides(&decl.name) {
            return expand_grid_line_shorthand(
                decl,
                start,
                end,
                specificity,
                origin,
                source_order,
                selector,
            );
        }
        if decl.name.eq_ignore_ascii_case("grid-area") {
            return expand_grid_area_shorthand(decl, specificity, origin, source_order, selector);
        }
        if let Some((align, justify)) = place_shorthand_sides(&decl.name) {
            return expand_place_shorthand(
                decl,
                align,
                justify,
                specificity,
                origin,
                source_order,
                selector,
                interner,
            );
        }
        vec![self.compile(decl, specificity, origin, source_order, selector, interner)]
    }
}

/// Returns the four longhand [`PropertyId`]s (in top/right/bottom/left order) a
/// `margin` / `padding` shorthand expands into, or [`None`] for any other name.
fn box_shorthand_sides(name: &str) -> Option<[PropertyId; 4]> {
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

/// Expands a `margin` / `padding` shorthand into its four longhand declarations.
///
/// The value list is lowered component-by-component into lengths (a unitless `0`
/// is accepted as `0px`); the CSS 1/2/3/4-value edge-replication rules then fan
/// the components out to the four sides. If any component is not a length, or the
/// list is empty or longer than four components, the whole shorthand degrades to
/// a single [`CompiledDeclaration::Unsupported`] rather than being dropped.
fn expand_box_shorthand(
    decl: &ParsedDeclaration,
    sides: [PropertyId; 4],
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
    interner: &mut CssSymbolInterner,
) -> Vec<CompiledDeclaration> {
    let unsupported = || {
        vec![CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
            value_unsupported_kind(&decl.value),
            &*decl.name,
            decl.span,
        ))]
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

/// The three longhand ids of one border edge, in `width` / `style` / `color`
/// order — the sub-properties the `border` and `border-<side>` shorthands set.
struct BorderSide {
    /// The edge's `border-*-width` longhand.
    width: PropertyId,
    /// The edge's `border-*-style` longhand.
    style: PropertyId,
    /// The edge's `border-*-color` longhand.
    color: PropertyId,
}

/// The four border edges in top/right/bottom/left order (the target of the
/// `border` shorthand and the per-edge fan-out of `border-width` / `-style` /
/// `-color`).
const BORDER_SIDES: &[BorderSide] = &[
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

/// Returns the single [`BorderSide`] a `border-top` / `-right` / `-bottom` /
/// `-left` shorthand targets, or [`None`] for any other name.
fn border_side_shorthand(name: &str) -> Option<&'static [BorderSide]> {
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

/// The `width` / `style` / `color` sub-values parsed from a `border` /
/// `border-<side>` shorthand value, each present only when authored.
struct BorderComponents {
    width: Option<PropertyValue>,
    style: Option<PropertyValue>,
    color: Option<PropertyValue>,
}

/// Classifies the components of a `border` / `border-<side>` value into its
/// `width` / `style` / `color` sub-values, in any order (spec CSS Backgrounds
/// and Borders §4).
///
/// Each component is a `<line-width>` (a length, or unitless `0`), a
/// `<line-style>` keyword ([`layout_keyword::border_style_from_name`]), or a
/// `<color>` (hex, `rgb()`/`rgba()`, or a named color). A component that fits
/// none of the three, or a second component of a category already seen, makes the
/// whole shorthand unparseable ([`None`]). An all-absent (empty) value is [`Some`]
/// with every field [`None`]; the caller reports that as unsupported.
fn classify_border(values: &[ParsedValue], interner: &mut CssSymbolInterner) -> Option<BorderComponents> {
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

/// Lowers one `border` component as a `<line-width>`: a `<length>` or a unitless
/// `0` (`0px`). Percentages and keywords are not border widths and yield [`None`].
fn border_width_component(value: &ParsedValue) -> Option<PropertyValue> {
    match value {
        ParsedValue::Number(n) if *n == 0.0 => Some(PropertyValue::Length(Length::px(0.0))),
        ParsedValue::Dimension { value, unit } => {
            dimension_length(*value, unit).map(PropertyValue::Length)
        }
        _ => None,
    }
}

/// Lowers one `border` component as a `<color>`: a hex hash, an `rgb()`/`rgba()`
/// function, or a modelled named color. Anything else yields [`None`].
fn border_color_component(
    value: &ParsedValue,
    _interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    match value {
        ParsedValue::Hash(hex) => parse_hex_color(hex).map(PropertyValue::Color),
        ParsedValue::Keyword(keyword) => named_color(keyword).map(PropertyValue::Color),
        ParsedValue::Function { name, args }
            if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") =>
        {
            parse_rgb(args).map(PropertyValue::Color)
        }
        _ => None,
    }
}

/// Expands a `border` / `border-<side>` shorthand into the present
/// `width` / `style` / `color` longhands for each targeted edge (spec CSS
/// Backgrounds and Borders §4).
///
/// The value is classified once into its sub-values ([`classify_border`]); each
/// authored sub-value is then emitted for every edge in `sides` (all four for
/// `border`, one for `border-top` / …). Omitted sub-values are **not** emitted —
/// a documented simplification of CSS's reset-to-initial semantics — so
/// `border: 2px solid red` sets width + style + color on every side while
/// `border-top: 1px solid red` sets them on the top edge only. An unparseable or
/// empty value degrades the whole shorthand to a single
/// [`CompiledDeclaration::Unsupported`].
#[allow(clippy::too_many_arguments)]
fn expand_border(
    decl: &ParsedDeclaration,
    sides: &[BorderSide],
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
    interner: &mut CssSymbolInterner,
) -> Vec<CompiledDeclaration> {
    let unsupported = || {
        vec![CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
            value_unsupported_kind(&decl.value),
            &*decl.name,
            decl.span,
        ))]
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

/// Expands a `border-color` shorthand into the four `border-*-color` longhands,
/// using the same 1/2/3/4-value edge replication as `margin` / `padding`.
///
/// Each component is a single `<color>` (hex, `rgb()`/`rgba()`, or a named
/// color); an unparseable component, or an empty or over-long list, degrades the
/// whole shorthand to a single [`CompiledDeclaration::Unsupported`].
fn expand_border_color(
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
    emit_four(decl, ids, &components, specificity, origin, source_order, selector)
}

/// Expands a `border-style` shorthand into the four `border-*-style` longhands,
/// with the same 1/2/3/4-value edge replication as `margin` / `padding`.
///
/// Each component is a `<line-style>` keyword folded to its reserved handle
/// (`none` / `solid`, via [`layout_keyword::border_style_from_name`]); an
/// unrecognised keyword, or an empty or over-long list, degrades the whole
/// shorthand to a single [`CompiledDeclaration::Unsupported`].
fn expand_border_style(
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
    emit_four(decl, ids, &components, specificity, origin, source_order, selector)
}

/// The single unsupported-feature fallback for a malformed `border-color` /
/// `border-style` value.
fn border_side_unsupported(decl: &ParsedDeclaration) -> Vec<CompiledDeclaration> {
    vec![CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
        value_unsupported_kind(&decl.value),
        &*decl.name,
        decl.span,
    ))]
}

/// Fans 1-4 pre-lowered `components` out to the four `ids` (top/right/bottom/left)
/// using the CSS edge-replication rules, emitting one [`Declaration`] per side.
/// An empty or over-long component list degrades to a single unsupported feature.
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

/// Returns the two longhand [`PropertyId`]s (`align`, `justify`) that a
/// `place-items` / `place-self` / `place-content` shorthand expands into, or
/// [`None`] for any other name. The CSS `place-*` grammar is `<align> <justify>`,
/// so the *align* side comes first (`place-content: <align-content>
/// <justify-content>`).
fn place_shorthand_sides(name: &str) -> Option<(PropertyId, PropertyId)> {
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

/// Expands a `place-items` / `place-self` shorthand into its `align-*` /
/// `justify-*` longhands (spec CSS Box Alignment).
///
/// The value is `<align> [<justify>]`: with a single component both longhands
/// take it; with two, the FIRST is the align side and the SECOND the justify
/// side (`place-items: center start` → `align-items: center; justify-items:
/// start`). Each side is lowered through its own longhand rules (so `place-self`
/// accepts `auto` while `place-items` does not, and a CSS-wide keyword like
/// `inherit` still resolves). A side that does not lower, or a value that is
/// empty or longer than two components, degrades the whole shorthand to a single
/// [`CompiledDeclaration::Unsupported`].
#[allow(clippy::too_many_arguments)]
fn expand_place_shorthand(
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
        vec![CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
            value_unsupported_kind(&decl.value),
            &*decl.name,
            decl.span,
        ))]
    };
    // The two sides' single-component value slices (align first, then justify).
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

/// Returns the two longhand [`PropertyId`]s (`start`, `end`) that the
/// `grid-column` / `grid-row` shorthand expands into, or [`None`] for any other
/// name.
fn grid_line_shorthand_sides(name: &str) -> Option<(PropertyId, PropertyId)> {
    if name.eq_ignore_ascii_case("grid-column") {
        Some((PropertyId::GridColumnStart, PropertyId::GridColumnEnd))
    } else if name.eq_ignore_ascii_case("grid-row") {
        Some((PropertyId::GridRowStart, PropertyId::GridRowEnd))
    } else {
        None
    }
}

/// Expands a `grid-column` / `grid-row` shorthand into its `*-start` / `*-end`
/// longhands (spec §4.13).
///
/// The value is split on a `/` delimiter into a start edge and an optional end
/// edge; each side is a `<grid-line>` — an integer line index (`3`), `span N`, or
/// `auto`. With no `/`, the end edge is `auto` (the line-based single-value form;
/// named-ident copying is out of scope), so `grid-column: 2` is `2 / auto` and
/// `grid-column: span 2` is `span 2 / auto`. An unparseable edge degrades the whole
/// shorthand to a single [`CompiledDeclaration::Unsupported`] rather than dropping
/// it.
fn expand_grid_line_shorthand(
    decl: &ParsedDeclaration,
    start_id: PropertyId,
    end_id: PropertyId,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
) -> Vec<CompiledDeclaration> {
    let unsupported = || {
        vec![CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
            value_unsupported_kind(&decl.value),
            &*decl.name,
            decl.span,
        ))]
    };
    // A `/` separates the start edge from the end edge; more than one `/` is not a
    // valid two-value placement.
    let mut sides = decl.value.split(|v| matches!(v, ParsedValue::Delimiter('/')));
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

/// Expands the `grid-area` shorthand into its FOUR line longhands (spec §4.13).
///
/// The value is split on `/` into up to four `<grid-line>` edges in CSS order —
/// `row-start / column-start / row-end / column-end` — mapping to
/// [`PropertyId::GridRowStart`], [`PropertyId::GridColumnStart`],
/// [`PropertyId::GridRowEnd`], [`PropertyId::GridColumnEnd`]. Omitted edges follow
/// the CSS filling rules over the LINE-based forms only:
///
/// * a lone `<custom-ident>` (`grid-area: header`) is a **named-area** placement:
///   it is carried as a single [`PropertyId::GridArea`] marker the runtime
///   resolves against the item's container `grid-template-areas`, not expanded to
///   the four line longhands.
/// * `a` (a lone line number) → all four edges are `a`.
/// * `a / b` → `row-start=a, column-start=b`, both ends `auto`.
/// * `a / b / c` → `row-start=a, column-start=b, row-end=c`, `column-end=auto`.
/// * `a / b / c / d` → the four edges directly.
///
/// An unparseable edge, or more than four values, degrades the whole shorthand to
/// a single [`CompiledDeclaration::Unsupported`] rather than dropping it.
fn expand_grid_area_shorthand(
    decl: &ParsedDeclaration,
    specificity: SelectorSpecificity,
    origin: CascadeOrigin,
    source_order: u32,
    selector: Option<SelectorId>,
) -> Vec<CompiledDeclaration> {
    let unsupported = || {
        vec![CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
            value_unsupported_kind(&decl.value),
            &*decl.name,
            decl.span,
        ))]
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
    for group in decl.value.split(|v| matches!(v, ParsedValue::Delimiter('/'))) {
        match grid_line_edge(group) {
            Some(edge) => edges.push(edge),
            None => return unsupported(),
        }
    }
    // Fill omitted edges: `[row-start, column-start, row-end, column-end]`.
    // `GridLineValue` is no longer `Copy` (its `Named` variant owns a name), so
    // clone the reused edges.
    let [row_start, column_start, row_end, column_end] = match edges.as_slice() {
        [a] => [a.clone(), a.clone(), a.clone(), a.clone()],
        [a, b] => [a.clone(), b.clone(), GridLineValue::Auto, GridLineValue::Auto],
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

/// Lowers one `<grid-line>` edge from a component group: `auto`, a non-zero
/// integer line index (`3`, `-1`), `span N` (`N` a positive integer, in either
/// order `span 2` / `2 span`), or a `<custom-ident>` line name
/// (`sidebar-start`). Returns [`None`] for anything else.
fn grid_line_edge(group: &[ParsedValue]) -> Option<GridLineValue> {
    match group {
        [ParsedValue::Keyword(keyword)] if keyword.eq_ignore_ascii_case("auto") => {
            Some(GridLineValue::Auto)
        }
        // A lone `<custom-ident>` (not `auto`, not a CSS-wide keyword) is a named
        // line, resolved against the container's named lines in the runtime. `span`
        // alone is not a valid edge, so exclude it (a bare `span` is not a line).
        [ParsedValue::Keyword(keyword)]
            if css_wide_keyword(keyword).is_none()
                && !keyword.eq_ignore_ascii_case("span") =>
        {
            Some(GridLineValue::Named((**keyword).into()))
        }
        [one] => grid_line_integer(one).map(GridLineValue::Line),
        // `span N` / `N span`: exactly one `span` keyword and one positive integer.
        [a, b] => {
            let span = grid_line_span(a, b).or_else(|| grid_line_span(b, a))?;
            Some(GridLineValue::Span(span))
        }
        _ => None,
    }
}

/// Reads a non-zero integer line index from a single component (`3`, `-1`); a
/// zero or non-integral number is not a valid line and yields [`None`].
#[allow(clippy::cast_possible_truncation)]
fn grid_line_integer(value: &ParsedValue) -> Option<i32> {
    match value {
        ParsedValue::Number(n) if n.fract() == 0.0 && *n != 0.0 => Some(*n as i32),
        _ => None,
    }
}

/// Reads a `span N` count when `keyword` is `span` and `count` is a positive
/// integer, as a `u16`; [`None`] otherwise.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn grid_line_span(keyword: &ParsedValue, count: &ParsedValue) -> Option<u16> {
    let ParsedValue::Keyword(keyword) = keyword else {
        return None;
    };
    if !keyword.eq_ignore_ascii_case("span") {
        return None;
    }
    match count {
        ParsedValue::Number(n) if n.fract() == 0.0 && *n >= 1.0 => {
            Some((*n as u32).min(u32::from(u16::MAX)) as u16)
        }
        _ => None,
    }
}

/// Lowers a `grid-column-start` / `-end` / `grid-row-start` / `-end` longhand into
/// a [`PropertyValue::GridLine`]. The CSS-wide keywords (`inherit` / `initial` /
/// `unset`) take priority; otherwise the value is a single `<grid-line>` edge.
fn lower_grid_line(values: &[ParsedValue]) -> Option<PropertyValue> {
    if let [ParsedValue::Keyword(keyword)] = values
        && let Some(wide) = css_wide_keyword(keyword)
    {
        return Some(wide);
    }
    grid_line_edge(values).map(PropertyValue::GridLine)
}

/// Lowers a `grid-auto-flow` value into a single reserved
/// [`PropertyValue::Keyword`].
///
/// `grid-auto-flow` is a *multi-keyword* property: the axis (`row` / `column`)
/// and an optional `dense` packing keyword may appear together in either order
/// (`row dense`, `dense column`, …). The CSS-wide keywords (`inherit` /
/// `initial` / `unset`) take priority when they stand alone. Otherwise the
/// keyword tokens are joined and folded to a single reserved cross-crate handle
/// via [`layout_keyword::grid_auto_flow_from_name`] (`row dense` and `column
/// dense` each collapse to one symbol), so the runtime compares one keyword
/// identity. A value that is not a valid combination yields [`None`], degrading
/// to an unsupported-feature diagnostic rather than being dropped.
fn lower_grid_auto_flow(values: &[ParsedValue]) -> Option<PropertyValue> {
    // A lone CSS-wide keyword (`inherit` / `initial` / `unset`) wins.
    if let [ParsedValue::Keyword(keyword)] = values
        && let Some(wide) = css_wide_keyword(keyword)
    {
        return Some(wide);
    }
    // Otherwise the value is one or two bare axis / `dense` keywords.
    let mut tokens: Vec<&str> = Vec::new();
    for value in values {
        match value {
            ParsedValue::Keyword(keyword) => tokens.push(keyword),
            _ => return None,
        }
    }
    let joined = tokens.join(" ");
    layout_keyword::grid_auto_flow_from_name(&joined).map(PropertyValue::Keyword)
}

/// Lowers one component of a box shorthand to a [`PropertyValue::Length`].
///
/// Accepts the same lengths as the longhand path (`px`, `%`, `auto`) plus a
/// unitless `0` (treated as `0px`); anything else yields [`None`].
fn box_side_length(value: &ParsedValue, interner: &mut CssSymbolInterner) -> Option<PropertyValue> {
    if let ParsedValue::Number(n) = value
        && *n == 0.0
    {
        return Some(PropertyValue::Length(Length::px(0.0)));
    }
    match lower_component(value, interner) {
        Some(length @ PropertyValue::Length(_)) => Some(length),
        _ => None,
    }
}

/// Resolves a CSS property name to its [`PropertyId`], case-insensitively.
///
/// Matches against every modelled *longhand*'s canonical [`PropertyId::name`].
/// Returns [`None`] for any name outside that set — including the `margin` /
/// `padding` shorthands, which are expanded to longhands by
/// [`PropertyCompiler::compile_all`], and custom properties, which are handled
/// separately by their `--` prefix.
#[must_use]
pub(crate) fn property_id_from_name(name: &str) -> Option<PropertyId> {
    const ALL: [PropertyId; 69] = [
        PropertyId::Color,
        PropertyId::BackgroundColor,
        PropertyId::Opacity,
        PropertyId::Width,
        PropertyId::Height,
        PropertyId::Display,
        PropertyId::Visibility,
        PropertyId::Position,
        PropertyId::Top,
        PropertyId::Right,
        PropertyId::Bottom,
        PropertyId::Left,
        PropertyId::ZIndex,
        PropertyId::MarginTop,
        PropertyId::MarginRight,
        PropertyId::MarginBottom,
        PropertyId::MarginLeft,
        PropertyId::PaddingTop,
        PropertyId::PaddingRight,
        PropertyId::PaddingBottom,
        PropertyId::PaddingLeft,
        PropertyId::BorderTopWidth,
        PropertyId::BorderRightWidth,
        PropertyId::BorderBottomWidth,
        PropertyId::BorderLeftWidth,
        PropertyId::BorderTopStyle,
        PropertyId::BorderRightStyle,
        PropertyId::BorderBottomStyle,
        PropertyId::BorderLeftStyle,
        PropertyId::BorderTopColor,
        PropertyId::BorderRightColor,
        PropertyId::BorderBottomColor,
        PropertyId::BorderLeftColor,
        PropertyId::BorderTopLeftRadius,
        PropertyId::BorderTopRightRadius,
        PropertyId::BorderBottomRightRadius,
        PropertyId::BorderBottomLeftRadius,
        PropertyId::MinWidth,
        PropertyId::MinHeight,
        PropertyId::MaxWidth,
        PropertyId::MaxHeight,
        PropertyId::Gap,
        PropertyId::FlexBasis,
        PropertyId::FlexGrow,
        PropertyId::FlexShrink,
        PropertyId::FlexDirection,
        PropertyId::JustifyContent,
        PropertyId::AlignContent,
        PropertyId::AlignItems,
        PropertyId::JustifyItems,
        PropertyId::AlignSelf,
        PropertyId::JustifySelf,
        PropertyId::GridTemplateColumns,
        PropertyId::GridTemplateRows,
        PropertyId::GridTemplateAreas,
        PropertyId::GridAutoFlow,
        PropertyId::GridAutoColumns,
        PropertyId::GridAutoRows,
        PropertyId::GridColumnStart,
        PropertyId::GridColumnEnd,
        PropertyId::GridRowStart,
        PropertyId::GridRowEnd,
        PropertyId::FontSize,
        PropertyId::FontFamily,
        PropertyId::Direction,
        PropertyId::FontWeight,
        PropertyId::FontStyle,
        PropertyId::TextAlign,
        PropertyId::TextOverflow,
    ];
    ALL.into_iter()
        .find(|id| name.eq_ignore_ascii_case(id.name()))
}

/// Narrows a parser `f64` into the `f32` the style/motion domains use.
///
/// The tokenizer parses into `f64` for range while the presentation domains
/// store `f32`; this single checked-narrowing point keeps the (intentional)
/// truncation contained.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn narrow_f32(value: f64) -> f32 {
    value as f32
}

/// Lowers a value list for a specific property, applying property-aware rules.
///
/// The four closed-set layout-keyword properties (`display`, `flex-direction`,
/// `justify-content`, `align-items`) resolve their keyword to the *reserved*
/// cross-crate [`InternedSymbol`] from [`layout_keyword`] so the runtime layout
/// bridge can compare keyword identities without sharing this crate's interner.
/// CSS-wide keywords (`inherit` / `initial` / `unset`) still take priority, and
/// a keyword outside the property's closed set falls back to the ordinary intern
/// path so it round-trips rather than being dropped. Every other property uses
/// the generic [`lower_value`] path unchanged.
fn lower_for_property(
    property: PropertyId,
    values: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    if property == PropertyId::FontFamily {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_font_family(values);
    }
    if property == PropertyId::FontWeight {
        return lower_font_weight(values);
    }
    if property == PropertyId::ZIndex {
        return lower_z_index(values);
    }
    if matches!(
        property,
        PropertyId::GridColumnStart
            | PropertyId::GridColumnEnd
            | PropertyId::GridRowStart
            | PropertyId::GridRowEnd
    ) {
        return lower_grid_line(values);
    }
    if matches!(
        property,
        PropertyId::GridTemplateColumns | PropertyId::GridTemplateRows
    ) {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_grid_template(values);
    }
    if property == PropertyId::GridTemplateAreas {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_grid_template_areas(values);
    }
    if property == PropertyId::GridAutoFlow {
        return lower_grid_auto_flow(values);
    }
    if matches!(
        property,
        PropertyId::GridAutoColumns | PropertyId::GridAutoRows
    ) {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_grid_auto_tracks(values);
    }
    if let Some(mapper) = layout_keyword_mapper(property)
        && let Some(ParsedValue::Keyword(keyword)) = values.first()
    {
        if let Some(wide) = css_wide_keyword(keyword) {
            return Some(wide);
        }
        return Some(match mapper(keyword) {
            Some(symbol) => PropertyValue::Keyword(symbol),
            None => PropertyValue::Keyword(interner.intern(keyword)),
        });
    }
    // A unitless `0` is a valid length for any length-typed longhand — `0` means
    // `0px` (CSS Values §4.5.3, "the unit may be omitted for zero lengths"). The
    // generic `lower_value` path would otherwise lower it to a `PropertyValue::
    // Number`, which a length property ignores (so `top: 0` would read as unset
    // instead of `0px`, unlike `top: 0px` or the `inset: 0` shorthand which the
    // box-shorthand path already handles via `box_side_length`).
    if is_length_longhand(property)
        && let [ParsedValue::Number(n)] = values
        && *n == 0.0
    {
        return Some(PropertyValue::Length(Length::px(0.0)));
    }
    lower_value(values, interner)
}

/// Whether `property` takes a `<length>`/`<length-percentage>` value, for which a
/// unitless `0` is accepted as `0px`.
fn is_length_longhand(property: PropertyId) -> bool {
    matches!(
        property,
        PropertyId::Top
            | PropertyId::Right
            | PropertyId::Bottom
            | PropertyId::Left
            | PropertyId::Width
            | PropertyId::Height
            | PropertyId::MinWidth
            | PropertyId::MinHeight
            | PropertyId::MaxWidth
            | PropertyId::MaxHeight
            | PropertyId::MarginTop
            | PropertyId::MarginRight
            | PropertyId::MarginBottom
            | PropertyId::MarginLeft
            | PropertyId::PaddingTop
            | PropertyId::PaddingRight
            | PropertyId::PaddingBottom
            | PropertyId::PaddingLeft
            | PropertyId::BorderTopWidth
            | PropertyId::BorderRightWidth
            | PropertyId::BorderBottomWidth
            | PropertyId::BorderLeftWidth
            | PropertyId::BorderTopLeftRadius
            | PropertyId::BorderTopRightRadius
            | PropertyId::BorderBottomRightRadius
            | PropertyId::BorderBottomLeftRadius
            | PropertyId::Gap
            | PropertyId::FlexBasis
            | PropertyId::FontSize
    )
}

/// Lowers a `font-family` value list into an ordered
/// [`PropertyValue::FontFamilies`], preserving author order.
///
/// The component list is split on commas into one group per family. Within a
/// group, a single quoted string is taken verbatim (`"Helvetica Neue"`), while
/// one-or-more bare idents are joined with a single space (`Helvetica Neue`);
/// a lone generic keyword (`serif`, `sans-serif`, …) is kept as its own name so
/// the runtime can map it to a generic family. Empty groups (a stray or trailing
/// comma) are skipped. Returns [`None`] when no family name can be recovered, so
/// the declaration degrades to an unsupported-feature diagnostic rather than
/// being dropped silently.
fn lower_font_family(values: &[ParsedValue]) -> Option<PropertyValue> {
    let mut families: Vec<Box<str>> = Vec::new();
    for group in values.split(|value| matches!(value, ParsedValue::Comma)) {
        if let Some(name) = font_family_name(group) {
            families.push(name);
        }
    }
    if families.is_empty() {
        return None;
    }
    Some(PropertyValue::FontFamilies(families.into_boxed_slice()))
}

/// Builds one family name from a comma-delimited component group: a lone quoted
/// string verbatim, or the group's bare idents joined by single spaces. Returns
/// [`None`] for a group with no ident/string components.
fn font_family_name(group: &[ParsedValue]) -> Option<Box<str>> {
    let mut idents: Vec<&str> = Vec::new();
    for value in group {
        match value {
            // A quoted family name or a bare ident both contribute their text.
            ParsedValue::String(text) | ParsedValue::Keyword(text) => idents.push(text),
            _ => {}
        }
    }
    if idents.is_empty() {
        None
    } else {
        Some(idents.join(" ").into_boxed_str())
    }
}

/// Returns the keyword→reserved-symbol mapper for a closed-set layout property,
/// or [`None`] for any property whose keywords are interned normally.
fn layout_keyword_mapper(property: PropertyId) -> Option<fn(&str) -> Option<InternedSymbol>> {
    match property {
        PropertyId::Display => Some(layout_keyword::display_from_name),
        PropertyId::Position => Some(layout_keyword::position_from_name),
        PropertyId::FlexDirection => Some(layout_keyword::flex_direction_from_name),
        PropertyId::JustifyContent => Some(layout_keyword::justify_content_from_name),
        // `align-content` shares the `justify-content` distribution value grammar
        // (`start`/`end`/`center`/`space-*`).
        PropertyId::AlignContent => Some(layout_keyword::align_content_from_name),
        // `align-items` and the grid container's `justify-items` share the closed
        // start/end/center/stretch keyword set.
        PropertyId::AlignItems | PropertyId::JustifyItems => {
            Some(layout_keyword::align_items_from_name)
        }
        // The per-item `*-self` properties additionally accept `auto`.
        PropertyId::AlignSelf | PropertyId::JustifySelf => {
            Some(layout_keyword::align_self_from_name)
        }
        // The four `border-*-style` longhands share the closed `none` / `solid`
        // (and folded visible-style) keyword set.
        PropertyId::BorderTopStyle
        | PropertyId::BorderRightStyle
        | PropertyId::BorderBottomStyle
        | PropertyId::BorderLeftStyle => Some(layout_keyword::border_style_from_name),
        PropertyId::Direction => Some(layout_keyword::direction_from_name),
        PropertyId::FontStyle => Some(layout_keyword::font_style_from_name),
        PropertyId::TextAlign => Some(layout_keyword::text_align_from_name),
        PropertyId::TextOverflow => Some(layout_keyword::text_overflow_from_name),
        _ => None,
    }
}

/// Lowers a `font-weight` value on the CSS `1..=1000` weight axis.
///
/// A bare number is taken verbatim, and the absolute named keywords map to their
/// numeric equivalents (`normal` → 400, `bold` → 700), both as a
/// [`PropertyValue::Number`]. The *relative* keywords (`bolder` / `lighter`)
/// lower to the reserved [`layout_keyword`] handles
/// [`FONT_WEIGHT_BOLDER`](layout_keyword::FONT_WEIGHT_BOLDER) /
/// [`FONT_WEIGHT_LIGHTER`](layout_keyword::FONT_WEIGHT_LIGHTER) instead of fixed
/// numbers, so the computed-value step can resolve them *relative* to the
/// parent's computed weight (§4.14). The CSS-wide keywords (`inherit` /
/// `initial` / `unset`) still take priority. Any other value yields [`None`] so
/// the declaration degrades to an unsupported-feature diagnostic.
fn lower_font_weight(values: &[ParsedValue]) -> Option<PropertyValue> {
    match values.first()? {
        ParsedValue::Number(n) => Some(PropertyValue::Number(narrow_f32(*n))),
        ParsedValue::Keyword(keyword) => {
            if let Some(wide) = css_wide_keyword(keyword) {
                return Some(wide);
            }
            if let Some(symbol) = layout_keyword::font_weight_relative_from_name(keyword) {
                return Some(PropertyValue::Keyword(symbol));
            }
            font_weight_keyword(keyword).map(PropertyValue::Number)
        }
        _ => None,
    }
}

/// Lowers a `z-index` value: an `<integer>` or the `auto` keyword (spec §5.1).
///
/// A bare integer is taken verbatim as a [`PropertyValue::Number`] (negative
/// values allowed, e.g. `z-index: -1`); a non-integral number is not a valid
/// `z-index` and yields [`None`]. The `auto` keyword lowers to the reserved
/// [`layout_keyword::Z_INDEX_AUTO`] handle so the runtime reads one keyword
/// identity across crates. The CSS-wide keywords (`inherit` / `initial` /
/// `unset`) still take priority. Any other value yields [`None`] so the
/// declaration degrades to an unsupported-feature diagnostic.
fn lower_z_index(values: &[ParsedValue]) -> Option<PropertyValue> {
    match values.first()? {
        ParsedValue::Number(n) if n.fract() == 0.0 => Some(PropertyValue::Number(narrow_f32(*n))),
        ParsedValue::Keyword(keyword) => {
            if let Some(wide) = css_wide_keyword(keyword) {
                return Some(wide);
            }
            layout_keyword::z_index_from_name(keyword).map(PropertyValue::Keyword)
        }
        _ => None,
    }
}

/// Maps an absolute named `font-weight` keyword to its numeric weight,
/// case-insensitively; [`None`] for any other keyword (including the relative
/// `bolder` / `lighter`, which lower to reserved handles).
fn font_weight_keyword(keyword: &str) -> Option<f32> {
    match keyword.to_ascii_lowercase().as_str() {
        "normal" => Some(400.0),
        "bold" => Some(700.0),
        _ => None,
    }
}

/// The largest number of tracks a single `repeat(N, …)` is allowed to expand to,
/// so a pathological repetition count cannot blow up the track list.
const MAX_REPEAT: usize = 1024;

/// Lowers a `grid-template-columns` / `grid-template-rows` value into a
/// [`PropertyValue::GridTracks`] track list (spec §4.1).
///
/// The value is a space-separated list of tracks; each track is a `<length>`
/// (`px`/`%`/`em`/… → [`GridTrack::Fixed`]), an `<fr>` (`1fr` → [`GridTrack::Fr`]),
/// the `auto` keyword ([`GridTrack::Auto`]), or `minmax(min, max)`
/// ([`GridTrack::MinMax`]). A `repeat(N, <tracks>)` is expanded in place into `N`
/// flat copies of its inner track list. Returns [`None`] (so the declaration
/// degrades to an unsupported-feature diagnostic) when the list is empty or any
/// component is not a recognised track.
fn lower_grid_template(values: &[ParsedValue]) -> Option<PropertyValue> {
    let (tracks, line_names) = track_list_from_components(values)?;
    if tracks.is_empty() {
        return None;
    }
    // A template with no authored `[name]` group carries an empty `line_names`,
    // keeping the value byte-identical to the pre-named-lines list; otherwise the
    // buckets are one per grid line (`tracks.len() + 1`).
    let line_names = if line_names.iter().all(Vec::is_empty) {
        Box::from([])
    } else {
        line_names
            .into_iter()
            .map(Vec::into_boxed_slice)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    };
    Some(PropertyValue::grid_track_list(GridTrackList::new(
        tracks.into_boxed_slice(),
        line_names,
    )))
}

/// The parsed track list of a `grid-template-*` value: the flat [`GridTrack`]
/// sizing functions plus one name bucket per grid line (`tracks.len() + 1`).
type ParsedTrackList = (Vec<GridTrack>, Vec<Vec<Box<str>>>);

/// Parses a `grid-template-columns` / `-rows` value into its flat [`GridTrack`]
/// list **and** the `[name]` names on each grid line (spec §4.1, §4.13).
///
/// This walks the same track grammar as [`tracks_from_components`], but also
/// threads a per-line name accumulator: a [`ParsedValue::LineNames`] group names
/// the line at the current boundary (before the next track), so
/// `[a] 100px [b] 1fr [c]` yields tracks `[100px, 1fr]` and line names
/// `[[a], [b], [c]]` — one bucket per grid line (`tracks.len() + 1`). Names
/// inside a `repeat(…)` are not tracked (its tracks expand, its names are
/// dropped) — a documented limitation. Returns [`None`] on any unparseable track.
fn track_list_from_components(values: &[ParsedValue]) -> Option<ParsedTrackList> {
    let mut tracks = Vec::new();
    // Start with the bucket for line 0 (before the first track).
    let mut line_names: Vec<Vec<Box<str>>> = vec![Vec::new()];
    for value in values {
        match value {
            // A comma has no meaning inside a track list; reject rather than drop.
            ParsedValue::Comma => return None,
            ParsedValue::LineNames(names) => {
                if let Some(bucket) = line_names.last_mut() {
                    bucket.extend(names.iter().cloned());
                }
            }
            ParsedValue::Function { name, args } if name.eq_ignore_ascii_case("repeat") => {
                for track in expand_repeat(args)? {
                    tracks.push(track);
                    line_names.push(Vec::new());
                }
            }
            other => {
                tracks.push(grid_track_scalar(other)?);
                line_names.push(Vec::new());
            }
        }
    }
    Some((tracks, line_names))
}

/// Lowers a `grid-auto-columns` / `grid-auto-rows` value into a
/// [`PropertyValue::GridTracks`] track list — the implicit-track sizing functions.
///
/// The track grammar is identical to `grid-template-*` (a `<length>`, an `<fr>`,
/// `auto`, or `minmax(min, max)`, with an integer `repeat(N, …)` expanded in
/// place), and it *reuses* [`tracks_from_components`]. The one difference: an
/// `auto-fill` / `auto-fit` `repeat(…)` is **invalid** for auto-tracks (there is
/// no container basis for a repetition count here), so a lowered
/// [`GridTrack::AutoRepeat`] degrades the declaration to an unsupported-feature
/// diagnostic (returns [`None`]) rather than being sized against a meaningless
/// repeat. Returns [`None`] on an empty list, exactly like `grid-template-*`.
fn lower_grid_auto_tracks(values: &[ParsedValue]) -> Option<PropertyValue> {
    let tracks = tracks_from_components(values)?;
    if tracks.is_empty() {
        return None;
    }
    // `auto-fill` / `auto-fit` is only valid on `grid-template-*`; reject it here.
    if tracks.iter().any(|t| matches!(t, GridTrack::AutoRepeat { .. })) {
        return None;
    }
    Some(PropertyValue::grid_tracks(tracks))
}

/// Lowers a `grid-template-areas` value into a [`PropertyValue::GridAreas`]
/// (spec §4.2, the named-areas subset).
///
/// The value is a sequence of quoted strings, one per grid row; each string is a
/// whitespace-separated list of cell tokens, where a token is a `<custom-ident>`
/// area name or a run of one-or-more `.` characters denoting a null (empty) cell.
/// Every row must have the same number of cells, and every named area must form a
/// filled rectangle — both are enforced by
/// [`GridAreaTemplate::from_rows`](GridAreaTemplate), which returns [`None`] on a
/// ragged grid or a non-rectangular area, degrading the declaration to an
/// unsupported-feature diagnostic rather than laying out against a malformed grid.
/// A value that is not a list of strings (or an empty list) is likewise [`None`].
fn lower_grid_template_areas(values: &[ParsedValue]) -> Option<PropertyValue> {
    let mut rows: Vec<Vec<Option<Box<str>>>> = Vec::new();
    for value in values {
        let ParsedValue::String(row) = value else {
            return None;
        };
        let cells: Vec<Option<Box<str>>> = row
            .split_whitespace()
            .map(|cell| {
                // A run of one-or-more `.` is a null cell; any other token is a name.
                if cell.bytes().all(|b| b == b'.') {
                    None
                } else {
                    Some(cell.into())
                }
            })
            .collect();
        rows.push(cells);
    }
    GridAreaTemplate::from_rows(&rows).map(PropertyValue::GridAreas)
}

/// Parses a space-separated component list into a flat [`GridTrack`] list,
/// expanding any `repeat(…)` component in place.
fn tracks_from_components(values: &[ParsedValue]) -> Option<Vec<GridTrack>> {
    let mut out = Vec::new();
    for value in values {
        match value {
            // A comma has no meaning inside a track list; reject rather than drop.
            ParsedValue::Comma => return None,
            // `[name]` line-name groups carry no sizing; they are captured by
            // `track_list_from_components` on the `grid-template-*` path and simply
            // skipped here (the `grid-auto-*` / inner-`repeat` paths keep no names).
            ParsedValue::LineNames(_) => {}
            ParsedValue::Function { name, args } if name.eq_ignore_ascii_case("repeat") => {
                out.extend(expand_repeat(args)?);
            }
            other => out.push(grid_track_scalar(other)?),
        }
    }
    Some(out)
}

/// Lowers one non-`repeat` track: `auto`, an `<fr>`, a `<length>`, or
/// `minmax(min, max)`. Returns [`None`] for anything else.
fn grid_track_scalar(value: &ParsedValue) -> Option<GridTrack> {
    match value {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto") => {
            Some(GridTrack::Auto)
        }
        ParsedValue::Dimension { value, unit } if unit.eq_ignore_ascii_case("fr") => {
            Some(GridTrack::Fr(narrow_f32(*value)))
        }
        ParsedValue::Dimension { value, unit } => {
            dimension_length(*value, unit).map(GridTrack::Fixed)
        }
        ParsedValue::Percentage(p) => Some(GridTrack::Fixed(Length::Percent(narrow_f32(*p)))),
        ParsedValue::Function { name, args } if name.eq_ignore_ascii_case("minmax") => {
            lower_minmax(args)
        }
        _ => None,
    }
}

/// Lowers `minmax(min, max)` into a [`GridTrack::MinMax`]; each bound is a single
/// non-`repeat` track. Returns [`None`] unless exactly two comma-separated,
/// single-component bounds are present.
fn lower_minmax(args: &[ParsedValue]) -> Option<GridTrack> {
    let groups: Vec<&[ParsedValue]> = args.split(|v| matches!(v, ParsedValue::Comma)).collect();
    let [min_group, max_group] = groups.as_slice() else {
        return None;
    };
    let min = grid_track_scalar(single_component(min_group)?)?;
    let max = grid_track_scalar(single_component(max_group)?)?;
    Some(GridTrack::minmax(min, max))
}

/// Expands a `repeat(…)` component into [`GridTrack`]s.
///
/// An integer `repeat(N, <tracks>)` is expanded in place into `N` flat copies of
/// its inner track list (`N` a positive integer clamped to [`MAX_REPEAT`]). A
/// `repeat(auto-fill | auto-fit, <tracks>)` cannot be resolved without the
/// container size, so it is carried symbolically as a single
/// [`GridTrack::AutoRepeat`] (the layout algorithm computes the repetition count
/// from the container's content extent); the `auto-fill` / `auto-fit` distinction
/// is preserved in its `fill` flag. Returns [`None`] on a missing count, an empty
/// inner list, or an unparseable inner track.
fn expand_repeat(args: &[ParsedValue]) -> Option<Vec<GridTrack>> {
    let comma = args.iter().position(|v| matches!(v, ParsedValue::Comma))?;
    let (count_part, rest) = args.split_at(comma);
    let inner = tracks_from_components(&rest[1..])?;
    if inner.is_empty() {
        return None;
    }
    // `auto-fill` / `auto-fit`: keep the pattern as an auto-repeat resolved later.
    if let Some(fill) = auto_repeat_fill(count_part) {
        return Some(vec![GridTrack::AutoRepeat {
            fill,
            tracks: inner.into_boxed_slice(),
        }]);
    }
    let count = repeat_count(count_part)?;
    let mut out = Vec::with_capacity(inner.len().saturating_mul(count));
    for _ in 0..count {
        out.extend(inner.iter().cloned());
    }
    Some(out)
}

/// Reads the `auto-fill` / `auto-fit` keyword of a `repeat(…)` count: `Some(true)`
/// for `auto-fill`, `Some(false)` for `auto-fit`, and [`None`] for any other count
/// (a numeric repetition or an invalid keyword).
fn auto_repeat_fill(part: &[ParsedValue]) -> Option<bool> {
    match single_component(part)? {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto-fill") => Some(true),
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("auto-fit") => Some(false),
        _ => None,
    }
}

/// Resolves the numeric count component of an integer `repeat(N, …)`: a positive
/// integer clamped to [`MAX_REPEAT`]. [`None`] for any non-numeric count (the
/// `auto-fill` / `auto-fit` keywords are handled by [`auto_repeat_fill`]).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn repeat_count(part: &[ParsedValue]) -> Option<usize> {
    match single_component(part)? {
        ParsedValue::Number(n) if *n >= 1.0 => Some((*n as usize).min(MAX_REPEAT)),
        _ => None,
    }
}

/// Returns the sole component of a group, or [`None`] when it has zero or more
/// than one component.
fn single_component(group: &[ParsedValue]) -> Option<&ParsedValue> {
    match group {
        [one] => Some(one),
        _ => None,
    }
}

/// Maps a CSS-wide keyword (`inherit` / `initial` / `unset`) to its
/// [`PropertyValue`], case-insensitively; [`None`] for any other keyword.
fn css_wide_keyword(keyword: &str) -> Option<PropertyValue> {
    if keyword.eq_ignore_ascii_case("inherit") {
        Some(PropertyValue::Inherit)
    } else if keyword.eq_ignore_ascii_case("initial") {
        Some(PropertyValue::Initial)
    } else if keyword.eq_ignore_ascii_case("unset") {
        Some(PropertyValue::Unset)
    } else {
        None
    }
}

/// Lowers a component-value list into a single [`PropertyValue`].
///
/// Only the first component is inspected: the modelled single-valued properties
/// take one component, and multi-component shorthands are handled separately by
/// [`expand_box_shorthand`]. Returns [`None`] when the component cannot be
/// lowered.
fn lower_value(values: &[ParsedValue], interner: &mut CssSymbolInterner) -> Option<PropertyValue> {
    values.first().and_then(|v| lower_component(v, interner))
}

/// Lowers one component value into a [`PropertyValue`], or [`None`].
fn lower_component(value: &ParsedValue, interner: &mut CssSymbolInterner) -> Option<PropertyValue> {
    match value {
        ParsedValue::Keyword(keyword) => Some(lower_keyword(keyword, interner)),
        ParsedValue::Number(n) => Some(PropertyValue::Number(narrow_f32(*n))),
        ParsedValue::Percentage(p) => Some(PropertyValue::Length(Length::Percent(narrow_f32(*p)))),
        ParsedValue::Dimension { value, unit } => lower_dimension(*value, unit),
        ParsedValue::Hash(hex) => parse_hex_color(hex).map(PropertyValue::Color),
        ParsedValue::Function { name, args } => lower_function(name, args, interner),
        ParsedValue::String(_)
        | ParsedValue::Comma
        | ParsedValue::Delimiter(_)
        | ParsedValue::LineNames(_) => None,
    }
}

/// Lowers a bare keyword, handling the CSS-wide keywords, `auto`, and the two
/// named colours the domain knows; anything else becomes an interned keyword.
fn lower_keyword(keyword: &str, interner: &mut CssSymbolInterner) -> PropertyValue {
    if let Some(wide) = css_wide_keyword(keyword) {
        return wide;
    }
    if keyword.eq_ignore_ascii_case("auto") {
        return PropertyValue::Length(Length::Auto);
    }
    if let Some(color) = named_color(keyword) {
        return PropertyValue::Color(color);
    }
    PropertyValue::Keyword(interner.intern(keyword))
}

/// Maps a CSS named color to a [`StyleColor`], case-insensitively; [`None`] for
/// any keyword that is not one of the modelled names.
///
/// This is the small named-color subset the domain recognises — `transparent`,
/// `black`, and the common basic colors — enough for authored `color` /
/// `background-color` / `border-*-color` values. The full CSS named-color table
/// is a later extension; an unrecognised name falls through to an interned
/// keyword rather than being lowered to a color.
fn named_color(keyword: &str) -> Option<StyleColor> {
    let c = |r: f32, g: f32, b: f32| StyleColor::rgba(r, g, b, 1.0);
    match keyword.to_ascii_lowercase().as_str() {
        "transparent" => Some(StyleColor::TRANSPARENT),
        "black" => Some(StyleColor::BLACK),
        "white" => Some(c(1.0, 1.0, 1.0)),
        "red" => Some(c(1.0, 0.0, 0.0)),
        "green" => Some(c(0.0, 128.0 / 255.0, 0.0)),
        "lime" => Some(c(0.0, 1.0, 0.0)),
        "blue" => Some(c(0.0, 0.0, 1.0)),
        "yellow" => Some(c(1.0, 1.0, 0.0)),
        "cyan" | "aqua" => Some(c(0.0, 1.0, 1.0)),
        "magenta" | "fuchsia" => Some(c(1.0, 0.0, 1.0)),
        "gray" | "grey" => Some(c(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0)),
        "silver" => Some(c(192.0 / 255.0, 192.0 / 255.0, 192.0 / 255.0)),
        "maroon" => Some(c(128.0 / 255.0, 0.0, 0.0)),
        "olive" => Some(c(128.0 / 255.0, 128.0 / 255.0, 0.0)),
        "navy" => Some(c(0.0, 0.0, 128.0 / 255.0)),
        "teal" => Some(c(0.0, 128.0 / 255.0, 128.0 / 255.0)),
        "purple" => Some(c(128.0 / 255.0, 0.0, 128.0 / 255.0)),
        "orange" => Some(c(1.0, 165.0 / 255.0, 0.0)),
        _ => None,
    }
}

/// Lowers a dimension into a [`Length`] term, accepting `px` (absolute), `%`
/// (percentage), the font-relative units (`em` / `rem`) and the viewport units
/// (`vw` / `vh` / `vmin` / `vmax`). Any other unit is outside the domain's
/// [`Length`] model and yields [`None`].
///
/// The relative units carry their coefficient symbolically; they resolve to
/// concrete pixels at the layout/compute step against a `LengthContext`
/// (spec §4.5).
pub(crate) fn dimension_length(value: f64, unit: &str) -> Option<Length> {
    let n = narrow_f32(value);
    if unit.eq_ignore_ascii_case("px") {
        Some(Length::px(n))
    } else if unit == "%" {
        Some(Length::Percent(n))
    } else if unit.eq_ignore_ascii_case("em") {
        Some(Length::Em(n))
    } else if unit.eq_ignore_ascii_case("rem") {
        Some(Length::Rem(n))
    } else if unit.eq_ignore_ascii_case("vw") {
        Some(Length::Vw(n))
    } else if unit.eq_ignore_ascii_case("vh") {
        Some(Length::Vh(n))
    } else if unit.eq_ignore_ascii_case("vmin") {
        Some(Length::Vmin(n))
    } else if unit.eq_ignore_ascii_case("vmax") {
        Some(Length::Vmax(n))
    } else {
        None
    }
}

/// Lowers a dimension into a [`PropertyValue::Length`], via [`dimension_length`].
fn lower_dimension(value: f64, unit: &str) -> Option<PropertyValue> {
    dimension_length(value, unit).map(PropertyValue::Length)
}

/// Lowers a function value: `var(--name)` to a [`PropertyValue::CustomRef`],
/// `rgb()`/`rgba()` to a colour, and `calc()`/`min()`/`max()`/`clamp()` (and a
/// bare parenthesised group) to a [`PropertyValue::Calc`] symbolic length
/// expression (spec §4.13). Any other function is not modelled and yields
/// [`None`].
fn lower_function(
    name: &str,
    args: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    if name.eq_ignore_ascii_case("var") {
        return args.iter().find_map(|arg| match arg {
            ParsedValue::Keyword(k) if k.starts_with("--") => {
                Some(PropertyValue::CustomRef(interner.intern(k)))
            }
            _ => None,
        });
    }
    if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") {
        return parse_rgb(args).map(PropertyValue::Color);
    }
    lower_math_function(name, args).map(PropertyValue::Calc)
}

/// Parses `rgb()`/`rgba()` arguments into a [`StyleColor`].
///
/// Colour channels accept numbers (`0..=255`) or percentages (`0%..=100%`);
/// alpha, when present, is a `0..=1` number or a percentage. Commas and a `/`
/// separator are skipped. Requires at least three channels.
fn parse_rgb(args: &[ParsedValue]) -> Option<StyleColor> {
    let mut channels: Vec<(f64, bool)> = Vec::new();
    for arg in args {
        match arg {
            ParsedValue::Number(n) => channels.push((*n, false)),
            ParsedValue::Percentage(p) => channels.push((*p, true)),
            _ => {}
        }
    }
    if channels.len() < 3 {
        return None;
    }
    let color_channel = |(value, is_percent): (f64, bool)| {
        if is_percent {
            narrow_f32(value) / 100.0
        } else {
            narrow_f32(value) / 255.0
        }
    };
    let r = color_channel(channels[0]);
    let g = color_channel(channels[1]);
    let b = color_channel(channels[2]);
    let a = channels.get(3).map_or(1.0, |&(value, is_percent)| {
        if is_percent {
            narrow_f32(value) / 100.0
        } else {
            narrow_f32(value)
        }
    });
    Some(StyleColor::rgba(r, g, b, a))
}

/// Parses a 3/4/6/8-digit hex colour (without the leading `#`) into a
/// [`StyleColor`], returning [`None`] on a bad length or non-hex digit.
fn parse_hex_color(hex: &str) -> Option<StyleColor> {
    if hex.is_empty() || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let pair = |slice: &str| {
        u8::from_str_radix(slice, 16)
            .ok()
            .map(|v| f32::from(v) / 255.0)
    };
    let single = |byte: u8| {
        let doubled = [byte, byte];
        let text = std::str::from_utf8(&doubled).ok()?;
        u8::from_str_radix(text, 16)
            .ok()
            .map(|v| f32::from(v) / 255.0)
    };
    let bytes = hex.as_bytes();
    match hex.len() {
        3 => Some(StyleColor::rgba(
            single(bytes[0])?,
            single(bytes[1])?,
            single(bytes[2])?,
            1.0,
        )),
        4 => Some(StyleColor::rgba(
            single(bytes[0])?,
            single(bytes[1])?,
            single(bytes[2])?,
            single(bytes[3])?,
        )),
        6 => Some(StyleColor::rgba(
            pair(&hex[0..2])?,
            pair(&hex[2..4])?,
            pair(&hex[4..6])?,
            1.0,
        )),
        8 => Some(StyleColor::rgba(
            pair(&hex[0..2])?,
            pair(&hex[2..4])?,
            pair(&hex[4..6])?,
            pair(&hex[6..8])?,
        )),
        _ => None,
    }
}

/// Picks the unsupported-feature category to report when a value cannot be
/// lowered, based on the shape of its first component.
fn value_unsupported_kind(values: &[ParsedValue]) -> UnsupportedFeatureKind {
    match values.first() {
        Some(ParsedValue::Function { .. }) => UnsupportedFeatureKind::Function,
        Some(ParsedValue::Dimension { .. }) => UnsupportedFeatureKind::Unit,
        _ => UnsupportedFeatureKind::Property,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedRule, PropertyParser, StylesheetParser};
    use crate::source::CssSource;

    fn decl(text: &str) -> ParsedDeclaration {
        let src = CssSource::anonymous(text);
        PropertyParser::new().parse(&src).unwrap()
    }

    fn compile(text: &str) -> (CompiledDeclaration, CssSymbolInterner) {
        let mut interner = CssSymbolInterner::new();
        let parsed = decl(text);
        let result = PropertyCompiler::new().compile(
            &parsed,
            SelectorSpecificity::ZERO,
            CascadeOrigin::GlobalApp,
            0,
            None,
            &mut interner,
        );
        (result, interner)
    }

    fn standard(text: &str) -> (Declaration, CssSymbolInterner) {
        match compile(text) {
            (CompiledDeclaration::Standard(d), i) => (d, i),
            (other, _) => panic!("expected standard declaration, got {other:?}"),
        }
    }

    fn value_of(text: &str) -> PropertyValue {
        standard(text).0.value().clone()
    }

    #[test]
    fn unitless_zero_is_a_zero_length_for_length_longhands() {
        let zero = PropertyValue::Length(Length::px(0.0));
        // Positioning insets, box model, sizing, gap, flex-basis, font-size.
        assert_eq!(value_of("top: 0"), zero);
        assert_eq!(value_of("right: 0"), zero);
        assert_eq!(value_of("width: 0"), zero);
        assert_eq!(value_of("margin-top: 0"), zero);
        assert_eq!(value_of("padding-left: 0"), zero);
        assert_eq!(value_of("gap: 0"), zero);
        // An explicit `0px` is unchanged, and a non-length property keeps a
        // unitless number as a `Number` (so `opacity: 0` is not a length).
        assert_eq!(value_of("top: 0px"), zero);
        assert_eq!(value_of("opacity: 0"), PropertyValue::Number(0.0));
    }

    fn compile_all(text: &str) -> Vec<CompiledDeclaration> {
        let mut interner = CssSymbolInterner::new();
        let parsed = decl(text);
        PropertyCompiler::new().compile_all(
            &parsed,
            SelectorSpecificity::ZERO,
            CascadeOrigin::GlobalApp,
            0,
            None,
            &mut interner,
        )
    }

    fn standard_of(compiled: &CompiledDeclaration) -> &Declaration {
        match compiled {
            CompiledDeclaration::Standard(d) => d,
            other => panic!("expected standard declaration, got {other:?}"),
        }
    }

    /// Collects the `(PropertyId, PropertyValue)` pairs of a compiled shorthand's
    /// standard declarations, in emission order.
    fn pairs(compiled: &[CompiledDeclaration]) -> Vec<(PropertyId, PropertyValue)> {
        compiled
            .iter()
            .map(|c| {
                let d = standard_of(c);
                (d.property(), d.value().clone())
            })
            .collect()
    }

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
        assert!(got.contains(&(PropertyId::BorderTopWidth, PropertyValue::Length(Length::px(2.0)))));
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
                (PropertyId::BorderTopWidth, PropertyValue::Length(Length::px(1.0))),
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
                (PropertyId::BorderTopLeftRadius, PropertyValue::Length(Length::px(1.0))),
                (PropertyId::BorderTopRightRadius, PropertyValue::Length(Length::px(2.0))),
                (PropertyId::BorderBottomRightRadius, PropertyValue::Length(Length::px(3.0))),
                (PropertyId::BorderBottomLeftRadius, PropertyValue::Length(Length::px(4.0))),
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
        // The name resolves as a known longhand.
        assert_eq!(
            property_id_from_name("border-bottom-right-radius"),
            Some(PropertyId::BorderBottomRightRadius)
        );
    }

    #[test]
    fn border_width_and_style_shorthands_fan_out() {
        assert_eq!(
            pairs(&compile_all("border-width: 3px")),
            vec![
                (PropertyId::BorderTopWidth, PropertyValue::Length(Length::px(3.0))),
                (PropertyId::BorderRightWidth, PropertyValue::Length(Length::px(3.0))),
                (PropertyId::BorderBottomWidth, PropertyValue::Length(Length::px(3.0))),
                (PropertyId::BorderLeftWidth, PropertyValue::Length(Length::px(3.0))),
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
            PropertyValue::Keyword(layout_keyword::BORDER_STYLE_SOLID)
        );
    }

    #[test]
    fn resolves_known_property_names() {
        assert_eq!(property_id_from_name("color"), Some(PropertyId::Color));
        assert_eq!(
            property_id_from_name("background-color"),
            Some(PropertyId::BackgroundColor)
        );
        assert_eq!(property_id_from_name("WIDTH"), Some(PropertyId::Width));
        assert_eq!(property_id_from_name("margin"), None);
    }

    #[test]
    fn unknown_property_is_unsupported() {
        let (result, _) = compile("float: left");
        let CompiledDeclaration::Unsupported(feature) = result else {
            panic!("expected unsupported");
        };
        assert_eq!(feature.kind(), UnsupportedFeatureKind::Property);
        assert_eq!(feature.name(), "float");
    }

    #[test]
    fn lowers_named_and_hex_colors() {
        assert_eq!(
            value_of("color: black"),
            PropertyValue::Color(StyleColor::BLACK)
        );
        assert_eq!(
            value_of("color: transparent"),
            PropertyValue::Color(StyleColor::TRANSPARENT)
        );
        assert_eq!(
            value_of("color: #ff0000"),
            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        );
    }

    #[test]
    fn lowers_short_and_alpha_hex() {
        assert_eq!(
            value_of("color: #f00"),
            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        );
        assert_eq!(
            value_of("color: #00000080"),
            PropertyValue::Color(StyleColor::rgba(0.0, 0.0, 0.0, 128.0 / 255.0))
        );
    }

    #[test]
    fn lowers_rgb_and_rgba_functions() {
        assert_eq!(
            value_of("color: rgb(255, 0, 0)"),
            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        );
        assert_eq!(
            value_of("color: rgba(0, 0, 0, 0.5)"),
            PropertyValue::Color(StyleColor::rgba(0.0, 0.0, 0.0, 0.5))
        );
        assert_eq!(
            value_of("color: rgb(100%, 0%, 0%)"),
            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        );
    }

    #[test]
    fn lowers_px_length_and_percentage() {
        assert_eq!(
            value_of("width: 10px"),
            PropertyValue::Length(Length::px(10.0))
        );
        assert_eq!(
            value_of("width: 50%"),
            PropertyValue::Length(Length::Percent(50.0))
        );
    }

    #[test]
    fn lowers_auto_keyword_to_length() {
        assert_eq!(value_of("width: auto"), PropertyValue::Length(Length::Auto));
    }

    #[test]
    fn lowers_bare_number() {
        assert_eq!(value_of("opacity: 0.5"), PropertyValue::Number(0.5));
    }

    #[test]
    fn lowers_css_wide_keywords() {
        assert_eq!(value_of("color: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("color: initial"), PropertyValue::Initial);
        assert_eq!(value_of("color: UNSET"), PropertyValue::Unset);
    }

    #[test]
    fn lowers_plain_keyword_to_interned_symbol() {
        // `visibility` is not a closed-set layout property, so its keyword is
        // interned normally (unlike `display`, which maps to a reserved symbol).
        let (d, interner) = standard("visibility: hidden");
        let PropertyValue::Keyword(symbol) = d.value().clone() else {
            panic!("expected keyword");
        };
        assert_eq!(interner.resolve(symbol), Some("hidden"));
    }

    #[test]
    fn lowers_var_to_custom_ref() {
        let (d, interner) = standard("color: var(--brand)");
        let PropertyValue::CustomRef(symbol) = d.value().clone() else {
            panic!("expected custom ref");
        };
        assert_eq!(interner.resolve(symbol), Some("--brand"));
    }

    #[test]
    fn lowers_custom_property() {
        let (result, interner) = compile("--brand: #fff");
        let CompiledDeclaration::Custom(custom) = result else {
            panic!("expected custom property");
        };
        assert_eq!(interner.resolve(custom.name()), Some("--brand"));
        assert_eq!(
            custom.value(),
            &PropertyValue::Color(StyleColor::rgba(1.0, 1.0, 1.0, 1.0))
        );
    }

    #[test]
    fn important_flag_is_carried() {
        let (normal, _) = standard("color: red");
        assert_eq!(normal.important(), ImportantFlag::Normal);
        let (important, _) = standard("color: red !important");
        assert_eq!(important.important(), ImportantFlag::Important);
    }

    #[test]
    fn cascade_context_is_recorded() {
        let mut interner = CssSymbolInterner::new();
        let parsed = decl("width: 10px");
        let selector = SelectorId::from_raw(9);
        let result = PropertyCompiler::new().compile(
            &parsed,
            SelectorSpecificity::new(1, 2, 3),
            CascadeOrigin::Theme,
            7,
            Some(selector),
            &mut interner,
        );
        let CompiledDeclaration::Standard(d) = result else {
            panic!("expected standard");
        };
        assert_eq!(d.origin(), CascadeOrigin::Theme);
        assert_eq!(d.specificity(), SelectorSpecificity::new(1, 2, 3));
        assert_eq!(d.source_order(), 7);
        assert_eq!(d.selector(), Some(selector));
    }

    #[test]
    fn lowers_relative_and_viewport_units() {
        use obscur_style::property::LengthContext;
        // font 16, root 20, viewport 800x600.
        let ctx = LengthContext::new(16.0, 20.0, 800.0, 600.0);
        let px = |text: &str| match value_of(text) {
            PropertyValue::Length(length) => length.resolve_px(&ctx),
            other => panic!("expected length, got {other:?}"),
        };
        assert_eq!(px("width: 3em"), Some(48.0));
        assert_eq!(px("width: 2rem"), Some(40.0));
        assert_eq!(px("width: 5vw"), Some(40.0));
        assert_eq!(px("width: 10vh"), Some(60.0));
        assert_eq!(px("width: 10vmin"), Some(60.0));
        assert_eq!(px("width: 10vmax"), Some(80.0));
    }

    #[test]
    fn lowers_calc_to_a_resolvable_expression() {
        use obscur_style::property::LengthContext;
        let ctx = LengthContext::new(16.0, 20.0, 800.0, 600.0);
        let PropertyValue::Calc(expr) = value_of("width: calc(10px + 2em)") else {
            panic!("expected calc");
        };
        // 10 + 2*16 = 42.
        assert_eq!(expr.resolve(&ctx), Some(42.0));
    }

    #[test]
    fn lowers_clamp_to_a_resolvable_expression() {
        use obscur_style::property::LengthContext;
        let ctx = LengthContext::new(16.0, 20.0, 800.0, 600.0);
        let PropertyValue::Calc(expr) = value_of("width: clamp(10px, 5vw, 100px)") else {
            panic!("expected calc");
        };
        // 5vw = 40, within [10, 100].
        assert_eq!(expr.resolve(&ctx), Some(40.0));
    }

    #[test]
    fn math_functions_and_relative_units_are_not_unsupported() {
        // The regression guard for audit V4: none of these fire an
        // unsupported-feature diagnostic any longer.
        for text in [
            "width: 3em",
            "width: 2rem",
            "width: 5vw",
            "width: calc(100% - 20px)",
            "width: min(10px, 5vw)",
            "width: max(10px, 5vw)",
            "width: clamp(10px, 5vw, 100px)",
        ] {
            let (result, _) = compile(text);
            assert!(
                matches!(result, CompiledDeclaration::Standard(_)),
                "{text} should lower to a standard declaration, got {result:?}"
            );
        }
    }

    #[test]
    fn bad_hex_length_is_unsupported() {
        // `#12345` is neither 3/4/6/8 digits: the value cannot lower.
        let (result, _) = compile("color: #12345");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    #[test]
    fn full_declaration_end_to_end() {
        let src = CssSource::anonymous("a { color: #00ff00 }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Style(rule) = &sheet.rules[0] else {
            panic!("expected style rule");
        };
        let mut interner = CssSymbolInterner::new();
        let result = PropertyCompiler::new().compile(
            &rule.declarations[0],
            SelectorSpecificity::ZERO,
            CascadeOrigin::GlobalApp,
            0,
            None,
            &mut interner,
        );
        assert!(matches!(
            result,
            CompiledDeclaration::Standard(d)
                if *d.value() == PropertyValue::Color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0))
        ));
    }

    // ---- P4b: box-model / sizing / flex / font longhands ----

    #[test]
    fn resolves_new_longhand_names() {
        assert_eq!(
            property_id_from_name("margin-top"),
            Some(PropertyId::MarginTop)
        );
        assert_eq!(
            property_id_from_name("padding-left"),
            Some(PropertyId::PaddingLeft)
        );
        assert_eq!(property_id_from_name("min-width"), Some(PropertyId::MinWidth));
        assert_eq!(property_id_from_name("gap"), Some(PropertyId::Gap));
        assert_eq!(
            property_id_from_name("flex-basis"),
            Some(PropertyId::FlexBasis)
        );
        assert_eq!(property_id_from_name("font-size"), Some(PropertyId::FontSize));
        // Case-insensitive, matching the existing seven.
        assert_eq!(
            property_id_from_name("FLEX-DIRECTION"),
            Some(PropertyId::FlexDirection)
        );
        // Shorthands are not longhand ids.
        assert_eq!(property_id_from_name("margin"), None);
        assert_eq!(property_id_from_name("padding"), None);
    }

    #[test]
    fn lowers_length_longhands() {
        assert_eq!(
            value_of("margin-top: 8px"),
            PropertyValue::Length(Length::px(8.0))
        );
        assert_eq!(
            value_of("padding-left: 5%"),
            PropertyValue::Length(Length::Percent(5.0))
        );
        assert_eq!(
            value_of("min-width: 100px"),
            PropertyValue::Length(Length::px(100.0))
        );
        assert_eq!(value_of("gap: 12px"), PropertyValue::Length(Length::px(12.0)));
        assert_eq!(
            value_of("font-size: 14px"),
            PropertyValue::Length(Length::px(14.0))
        );
    }

    #[test]
    fn length_longhand_carries_its_property_id() {
        assert_eq!(
            standard("margin-top: 8px").0.property(),
            PropertyId::MarginTop
        );
        assert_eq!(standard("font-size: 14px").0.property(), PropertyId::FontSize);
    }

    #[test]
    fn flex_basis_auto_lowers_to_auto_length() {
        assert_eq!(
            value_of("flex-basis: auto"),
            PropertyValue::Length(Length::Auto)
        );
    }

    #[test]
    fn lowers_number_valued_flex_props() {
        assert_eq!(value_of("flex-grow: 1"), PropertyValue::Number(1.0));
        assert_eq!(value_of("flex-shrink: 0"), PropertyValue::Number(0.0));
    }

    #[test]
    fn grid_alignment_property_names_resolve() {
        assert_eq!(
            property_id_from_name("justify-items"),
            Some(PropertyId::JustifyItems)
        );
        assert_eq!(
            property_id_from_name("justify-self"),
            Some(PropertyId::JustifySelf)
        );
        assert_eq!(
            property_id_from_name("align-self"),
            Some(PropertyId::AlignSelf)
        );
    }

    #[test]
    fn grid_alignment_keywords_map_to_reserved_symbols() {
        // `justify-items` shares the `align-items` closed set (no `auto`).
        assert_eq!(
            value_of("justify-items: center"),
            PropertyValue::Keyword(layout_keyword::ALIGN_CENTER)
        );
        assert_eq!(
            value_of("justify-items: start"),
            PropertyValue::Keyword(layout_keyword::ALIGN_START)
        );
        // The `*-self` properties additionally accept `auto`.
        assert_eq!(
            value_of("justify-self: center"),
            PropertyValue::Keyword(layout_keyword::ALIGN_CENTER)
        );
        assert_eq!(
            value_of("align-self: end"),
            PropertyValue::Keyword(layout_keyword::ALIGN_END)
        );
        assert_eq!(
            value_of("align-self: auto"),
            PropertyValue::Keyword(layout_keyword::ALIGN_AUTO)
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
    fn align_content_maps_to_reserved_symbols() {
        // `align-content` shares the `justify-content` distribution grammar.
        assert_eq!(
            value_of("align-content: space-between"),
            PropertyValue::Keyword(layout_keyword::JUSTIFY_SPACE_BETWEEN)
        );
        assert_eq!(
            value_of("align-content: space-evenly"),
            PropertyValue::Keyword(layout_keyword::JUSTIFY_SPACE_EVENLY)
        );
        assert_eq!(
            value_of("align-content: center"),
            PropertyValue::Keyword(layout_keyword::JUSTIFY_CENTER)
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
    fn display_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("display: flex"),
            PropertyValue::Keyword(layout_keyword::DISPLAY_FLEX)
        );
        assert_eq!(
            value_of("display: grid"),
            PropertyValue::Keyword(layout_keyword::DISPLAY_GRID)
        );
        assert_eq!(
            value_of("display: block"),
            PropertyValue::Keyword(layout_keyword::DISPLAY_BLOCK)
        );
        assert_eq!(
            value_of("display: none"),
            PropertyValue::Keyword(layout_keyword::DISPLAY_NONE)
        );
    }

    #[test]
    fn flex_layout_keywords_map_to_reserved_symbols() {
        assert_eq!(
            value_of("flex-direction: column"),
            PropertyValue::Keyword(layout_keyword::FLEX_COLUMN)
        );
        assert_eq!(
            value_of("justify-content: center"),
            PropertyValue::Keyword(layout_keyword::JUSTIFY_CENTER)
        );
        assert_eq!(
            value_of("align-items: stretch"),
            PropertyValue::Keyword(layout_keyword::ALIGN_STRETCH)
        );
    }

    #[test]
    fn unknown_layout_keyword_falls_back_to_intern() {
        // `display: inline` is outside the closed set: it interns rather than
        // resolving to a reserved symbol (and is not dropped).
        let (d, interner) = standard("display: inline");
        let PropertyValue::Keyword(symbol) = d.value().clone() else {
            panic!("expected interned keyword");
        };
        assert_eq!(interner.resolve(symbol), Some("inline"));
    }

    #[test]
    fn layout_keyword_prop_honours_css_wide_keywords() {
        assert_eq!(value_of("display: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("flex-direction: initial"), PropertyValue::Initial);
    }

    #[test]
    fn z_index_lowers_integer_and_auto() {
        // `z-index` is resolved a longhand name.
        assert_eq!(property_id_from_name("z-index"), Some(PropertyId::ZIndex));
        // A positive integer lowers to a bare number.
        assert_eq!(value_of("z-index: 5"), PropertyValue::Number(5.0));
        // A negative integer is valid and lowers to a negative number.
        assert_eq!(value_of("z-index: -1"), PropertyValue::Number(-1.0));
        // `auto` lowers to the reserved cross-crate handle.
        assert_eq!(
            value_of("z-index: auto"),
            PropertyValue::Keyword(layout_keyword::Z_INDEX_AUTO)
        );
        // CSS-wide keywords still take priority.
        assert_eq!(value_of("z-index: inherit"), PropertyValue::Inherit);
    }

    #[test]
    fn z_index_non_integer_is_unsupported() {
        // A non-integral number is not a valid `z-index`; it degrades rather than
        // being dropped.
        let (result, _) = compile("z-index: 1.5");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    #[test]
    fn position_keyword_lowers_to_reserved_symbol() {
        assert_eq!(
            value_of("position: relative"),
            PropertyValue::Keyword(layout_keyword::POSITION_RELATIVE)
        );
        assert_eq!(
            value_of("position: static"),
            PropertyValue::Keyword(layout_keyword::POSITION_STATIC)
        );
        assert_eq!(
            value_of("position: absolute"),
            PropertyValue::Keyword(layout_keyword::POSITION_ABSOLUTE)
        );
        assert_eq!(
            value_of("position: fixed"),
            PropertyValue::Keyword(layout_keyword::POSITION_FIXED)
        );
        assert_eq!(
            value_of("position: sticky"),
            PropertyValue::Keyword(layout_keyword::POSITION_STICKY)
        );
    }

    #[test]
    fn inset_longhands_lower_to_lengths() {
        assert_eq!(value_of("top: 10px"), PropertyValue::Length(Length::px(10.0)));
        assert_eq!(
            value_of("left: 5px"),
            PropertyValue::Length(Length::px(5.0))
        );
        assert_eq!(
            value_of("right: 50%"),
            PropertyValue::Length(Length::Percent(50.0))
        );
        assert_eq!(value_of("bottom: auto"), PropertyValue::Length(Length::Auto));
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
    fn resolves_font_family_name() {
        assert_eq!(
            property_id_from_name("font-family"),
            Some(PropertyId::FontFamily)
        );
        assert_eq!(
            property_id_from_name("FONT-FAMILY"),
            Some(PropertyId::FontFamily)
        );
    }

    /// Extracts the ordered family names from a lowered `font-family` value.
    fn families(text: &str) -> Vec<String> {
        match value_of(text) {
            PropertyValue::FontFamilies(names) => {
                names.iter().map(|n| n.to_string()).collect()
            }
            other => panic!("expected font families, got {other:?}"),
        }
    }

    #[test]
    fn lowers_font_family_list_in_order() {
        // Quoted string, bare ident, and generic keyword, in author order.
        assert_eq!(
            families(r#"font-family: "Foo", Arial, sans-serif"#),
            vec![
                "Foo".to_string(),
                "Arial".to_string(),
                "sans-serif".to_string()
            ]
        );
    }

    #[test]
    fn lowers_multi_ident_family_joined_by_spaces() {
        // Unquoted multi-word family names join their idents with single spaces.
        assert_eq!(
            families("font-family: Helvetica Neue, monospace"),
            vec!["Helvetica Neue".to_string(), "monospace".to_string()]
        );
    }

    #[test]
    fn font_family_carries_its_property_id() {
        assert_eq!(
            standard("font-family: Arial").0.property(),
            PropertyId::FontFamily
        );
    }

    #[test]
    fn font_family_honours_css_wide_keywords() {
        assert_eq!(value_of("font-family: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("font-family: initial"), PropertyValue::Initial);
    }

    // ---- direction / font-weight / font-style ----

    #[test]
    fn resolves_direction_weight_style_names() {
        assert_eq!(property_id_from_name("direction"), Some(PropertyId::Direction));
        assert_eq!(
            property_id_from_name("FONT-WEIGHT"),
            Some(PropertyId::FontWeight)
        );
        assert_eq!(property_id_from_name("font-style"), Some(PropertyId::FontStyle));
    }

    #[test]
    fn direction_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("direction: rtl"),
            PropertyValue::Keyword(layout_keyword::DIRECTION_RTL)
        );
        assert_eq!(
            value_of("direction: ltr"),
            PropertyValue::Keyword(layout_keyword::DIRECTION_LTR)
        );
        assert_eq!(standard("direction: rtl").0.property(), PropertyId::Direction);
    }

    #[test]
    fn font_style_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("font-style: italic"),
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_ITALIC)
        );
        assert_eq!(
            value_of("font-style: normal"),
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_NORMAL)
        );
        assert_eq!(
            value_of("font-style: oblique"),
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_OBLIQUE)
        );
    }

    #[test]
    fn grid_auto_flow_maps_to_reserved_symbols() {
        assert_eq!(
            property_id_from_name("grid-auto-flow"),
            Some(PropertyId::GridAutoFlow)
        );
        assert_eq!(
            value_of("grid-auto-flow: row"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_ROW)
        );
        assert_eq!(
            value_of("grid-auto-flow: column"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_COLUMN)
        );
        // Bare `dense` folds to row-dense.
        assert_eq!(
            value_of("grid-auto-flow: dense"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_ROW_DENSE)
        );
        // The multi-token `column dense` folds to a single canonical symbol.
        assert_eq!(
            value_of("grid-auto-flow: column dense"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_COLUMN_DENSE)
        );
        assert_eq!(
            value_of("grid-auto-flow: row dense"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_ROW_DENSE)
        );
        assert_eq!(
            standard("grid-auto-flow: dense").0.property(),
            PropertyId::GridAutoFlow
        );
    }

    #[test]
    fn resolves_text_align_name() {
        assert_eq!(
            property_id_from_name("text-align"),
            Some(PropertyId::TextAlign)
        );
        assert_eq!(
            property_id_from_name("TEXT-ALIGN"),
            Some(PropertyId::TextAlign)
        );
    }

    #[test]
    fn text_align_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("text-align: justify"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_JUSTIFY)
        );
        assert_eq!(
            value_of("text-align: center"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_CENTER)
        );
        assert_eq!(
            value_of("text-align: right"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_RIGHT)
        );
        assert_eq!(
            value_of("text-align: left"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_LEFT)
        );
        assert_eq!(
            value_of("text-align: start"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_START)
        );
        assert_eq!(
            value_of("text-align: end"),
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_END)
        );
        assert_eq!(standard("text-align: justify").0.property(), PropertyId::TextAlign);
    }

    #[test]
    fn resolves_text_overflow_name() {
        assert_eq!(
            property_id_from_name("text-overflow"),
            Some(PropertyId::TextOverflow)
        );
        assert_eq!(
            property_id_from_name("TEXT-OVERFLOW"),
            Some(PropertyId::TextOverflow)
        );
    }

    #[test]
    fn text_overflow_maps_to_reserved_symbols() {
        assert_eq!(
            value_of("text-overflow: ellipsis"),
            PropertyValue::Keyword(layout_keyword::TEXT_OVERFLOW_ELLIPSIS)
        );
        assert_eq!(
            value_of("text-overflow: clip"),
            PropertyValue::Keyword(layout_keyword::TEXT_OVERFLOW_CLIP)
        );
        assert_eq!(
            standard("text-overflow: ellipsis").0.property(),
            PropertyId::TextOverflow
        );
        // CSS-wide keywords still take priority.
        assert_eq!(value_of("text-overflow: inherit"), PropertyValue::Inherit);
    }

    #[test]
    fn font_weight_lowers_numbers_and_keywords() {
        assert_eq!(value_of("font-weight: 300"), PropertyValue::Number(300.0));
        assert_eq!(value_of("font-weight: 600"), PropertyValue::Number(600.0));
        assert_eq!(value_of("font-weight: bold"), PropertyValue::Number(700.0));
        assert_eq!(value_of("font-weight: normal"), PropertyValue::Number(400.0));
        assert_eq!(standard("font-weight: 500").0.property(), PropertyId::FontWeight);
    }

    #[test]
    fn font_weight_relative_keywords_lower_to_reserved_handles() {
        // `bolder` / `lighter` are carried as reserved handles so the computed-
        // value step can resolve them relative to the parent (§4.14), rather than
        // being flattened to fixed numbers here.
        assert_eq!(
            value_of("font-weight: bolder"),
            PropertyValue::Keyword(layout_keyword::FONT_WEIGHT_BOLDER)
        );
        assert_eq!(
            value_of("font-weight: lighter"),
            PropertyValue::Keyword(layout_keyword::FONT_WEIGHT_LIGHTER)
        );
    }

    #[test]
    fn direction_weight_style_honour_css_wide_keywords() {
        assert_eq!(value_of("direction: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("font-weight: initial"), PropertyValue::Initial);
        assert_eq!(value_of("font-style: unset"), PropertyValue::Unset);
    }

    #[test]
    fn compile_all_wraps_a_single_longhand() {
        let decls = compile_all("width: 10px");
        assert_eq!(decls.len(), 1);
        assert_eq!(standard_of(&decls[0]).property(), PropertyId::Width);
    }

    // ---- grid-template-columns / grid-template-rows ----

    /// Extracts the lowered [`GridTrack`] list for a `grid-template-*` value.
    fn tracks(text: &str) -> Vec<GridTrack> {
        match value_of(text) {
            PropertyValue::GridTracks(list) => list.into_tracks().into_vec(),
            other => panic!("expected grid tracks, got {other:?}"),
        }
    }

    #[test]
    fn resolves_grid_template_names() {
        assert_eq!(
            property_id_from_name("grid-template-columns"),
            Some(PropertyId::GridTemplateColumns)
        );
        assert_eq!(
            property_id_from_name("GRID-TEMPLATE-ROWS"),
            Some(PropertyId::GridTemplateRows)
        );
    }

    #[test]
    fn lowers_fixed_and_fr_track_list() {
        assert_eq!(
            tracks("grid-template-columns: 100px 1fr 2fr"),
            vec![
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0),
                GridTrack::Fr(2.0),
            ]
        );
        // Carries its own property id, and rows lower identically.
        assert_eq!(
            standard("grid-template-columns: 1fr").0.property(),
            PropertyId::GridTemplateColumns
        );
        assert_eq!(
            standard("grid-template-rows: 1fr").0.property(),
            PropertyId::GridTemplateRows
        );
    }

    #[test]
    fn lowers_auto_and_percent_and_em_tracks() {
        assert_eq!(
            tracks("grid-template-columns: auto"),
            vec![GridTrack::Auto]
        );
        assert_eq!(
            tracks("grid-template-columns: 50% 2em"),
            vec![
                GridTrack::Fixed(Length::Percent(50.0)),
                GridTrack::Fixed(Length::Em(2.0)),
            ]
        );
    }

    #[test]
    fn lowers_repeat_to_flat_list() {
        assert_eq!(
            tracks("grid-template-columns: repeat(2, 1fr)"),
            vec![GridTrack::Fr(1.0), GridTrack::Fr(1.0)]
        );
        // A multi-track repeat body expands each copy in order.
        assert_eq!(
            tracks("grid-template-columns: repeat(2, 100px 1fr)"),
            vec![
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0),
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0),
            ]
        );
        // repeat mixes with surrounding tracks.
        assert_eq!(
            tracks("grid-template-columns: 20px repeat(2, 1fr)"),
            vec![
                GridTrack::Fixed(Length::px(20.0)),
                GridTrack::Fr(1.0),
                GridTrack::Fr(1.0),
            ]
        );
    }

    /// The full track list (tracks + line names) for a `grid-template-*` value.
    fn track_list(text: &str) -> GridTrackList {
        match value_of(text) {
            PropertyValue::GridTracks(list) => list,
            other => panic!("expected grid tracks, got {other:?}"),
        }
    }

    #[test]
    fn lowers_line_names_interleaved_with_tracks() {
        // `[a] 100px [b] 1fr [c]` -> tracks [Fixed(100), Fr(1)] with one name bucket
        // per grid line: [[a], [b], [c]].
        let list = track_list("grid-template-columns: [a] 100px [b] 1fr [c]");
        assert_eq!(
            list.tracks(),
            &[GridTrack::Fixed(Length::px(100.0)), GridTrack::Fr(1.0)]
        );
        let names: Vec<Vec<&str>> = list
            .line_names()
            .iter()
            .map(|b| b.iter().map(|n| &**n).collect())
            .collect();
        assert_eq!(names, vec![vec!["a"], vec!["b"], vec!["c"]]);
    }

    #[test]
    fn line_can_carry_multiple_names() {
        // `[sidebar-end main-start]` names one line twice.
        let list =
            track_list("grid-template-columns: [s-start] 200px [s-end main-start] 1fr [main-end]");
        let names: Vec<Vec<&str>> = list
            .line_names()
            .iter()
            .map(|b| b.iter().map(|n| &**n).collect())
            .collect();
        assert_eq!(
            names,
            vec![
                vec!["s-start"],
                vec!["s-end", "main-start"],
                vec!["main-end"]
            ]
        );
    }

    #[test]
    fn track_list_without_names_has_empty_line_names() {
        // A no-`[name]` template stays byte-identical: no line-name buckets.
        let list = track_list("grid-template-columns: 100px 1fr");
        assert!(list.line_names().is_empty());
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
    fn lowers_minmax_track() {
        assert_eq!(
            tracks("grid-template-columns: minmax(100px, 1fr)"),
            vec![GridTrack::minmax(
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0)
            )]
        );
    }

    #[test]
    fn auto_fill_repeat_lowers_to_auto_repeat() {
        // `auto-fill` cannot be resolved without a container width, so its pattern
        // is carried as a single `AutoRepeat` (not expanded to one repetition) and
        // the layout algorithm computes the count from the container size.
        assert_eq!(
            tracks("grid-template-columns: repeat(auto-fill, 100px)"),
            vec![GridTrack::AutoRepeat {
                fill: true,
                tracks: Box::from([GridTrack::Fixed(Length::px(100.0))]),
            }]
        );
    }

    #[test]
    fn auto_fit_repeat_lowers_to_auto_repeat() {
        assert_eq!(
            tracks("grid-template-columns: repeat(auto-fit, 100px 1fr)"),
            vec![GridTrack::AutoRepeat {
                fill: false,
                tracks: Box::from([GridTrack::Fixed(Length::px(100.0)), GridTrack::Fr(1.0)]),
            }]
        );
    }

    #[test]
    fn auto_fill_repeat_keeps_surrounding_tracks() {
        // A leading fixed track and a trailing track flank the auto-repeat.
        assert_eq!(
            tracks("grid-template-columns: 50px repeat(auto-fill, 100px) 20px"),
            vec![
                GridTrack::Fixed(Length::px(50.0)),
                GridTrack::AutoRepeat {
                    fill: true,
                    tracks: Box::from([GridTrack::Fixed(Length::px(100.0))]),
                },
                GridTrack::Fixed(Length::px(20.0)),
            ]
        );
    }

    #[test]
    fn grid_template_honours_css_wide_keywords() {
        assert_eq!(
            value_of("grid-template-columns: inherit"),
            PropertyValue::Inherit
        );
        assert_eq!(
            value_of("grid-template-rows: initial"),
            PropertyValue::Initial
        );
    }

    #[test]
    fn invalid_grid_template_is_unsupported() {
        // A bare colour is not a valid track.
        let (result, _) = compile("grid-template-columns: red");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    // ---- grid-auto-columns / grid-auto-rows ----

    #[test]
    fn resolves_grid_auto_track_names() {
        assert_eq!(
            property_id_from_name("grid-auto-columns"),
            Some(PropertyId::GridAutoColumns)
        );
        assert_eq!(
            property_id_from_name("GRID-AUTO-ROWS"),
            Some(PropertyId::GridAutoRows)
        );
    }

    #[test]
    fn lowers_grid_auto_rows_track_list() {
        // `grid-auto-rows: 40px` -> a single fixed 40px implicit-track size.
        assert_eq!(
            tracks("grid-auto-rows: 40px"),
            vec![GridTrack::Fixed(Length::px(40.0))]
        );
        assert_eq!(
            standard("grid-auto-rows: 40px").0.property(),
            PropertyId::GridAutoRows
        );
        // Cycled multi-track and minmax reuse the same track grammar.
        assert_eq!(
            tracks("grid-auto-columns: 30px 50px"),
            vec![GridTrack::Fixed(Length::px(30.0)), GridTrack::Fixed(Length::px(50.0))]
        );
        assert_eq!(
            tracks("grid-auto-columns: minmax(50px, 1fr)"),
            vec![GridTrack::minmax(
                GridTrack::Fixed(Length::px(50.0)),
                GridTrack::Fr(1.0)
            )]
        );
    }

    #[test]
    fn grid_auto_tracks_honour_css_wide_keywords() {
        assert_eq!(value_of("grid-auto-rows: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("grid-auto-columns: initial"), PropertyValue::Initial);
    }

    #[test]
    fn grid_auto_tracks_reject_auto_repeat() {
        // `auto-fill` / `auto-fit` is invalid for implicit tracks (no container
        // basis for a repetition count); the declaration is unsupported.
        let (result, _) = compile("grid-auto-columns: repeat(auto-fill, 100px)");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    // ---- grid-column / grid-row line placement ----

    /// The `(PropertyId, GridLineValue)` pairs a compiled declaration list carries.
    fn grid_lines(compiled: &[CompiledDeclaration]) -> Vec<(PropertyId, GridLineValue)> {
        compiled
            .iter()
            .map(|d| {
                let decl = standard_of(d);
                let PropertyValue::GridLine(edge) = decl.value() else {
                    panic!("expected grid line value, got {:?}", decl.value());
                };
                (decl.property(), edge.clone())
            })
            .collect()
    }

    #[test]
    fn resolves_grid_line_longhand_names() {
        assert_eq!(
            property_id_from_name("grid-column-start"),
            Some(PropertyId::GridColumnStart)
        );
        assert_eq!(
            property_id_from_name("GRID-COLUMN-END"),
            Some(PropertyId::GridColumnEnd)
        );
        assert_eq!(
            property_id_from_name("grid-row-start"),
            Some(PropertyId::GridRowStart)
        );
        assert_eq!(
            property_id_from_name("grid-row-end"),
            Some(PropertyId::GridRowEnd)
        );
        // The shorthands are not longhand ids.
        assert_eq!(property_id_from_name("grid-column"), None);
        assert_eq!(property_id_from_name("grid-row"), None);
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
    fn grid_line_auto_edge_is_accepted() {
        assert_eq!(
            grid_lines(&compile_all("grid-column: auto / 3")),
            vec![
                (PropertyId::GridColumnStart, GridLineValue::Auto),
                (PropertyId::GridColumnEnd, GridLineValue::Line(3)),
            ]
        );
    }

    #[test]
    fn grid_line_longhands_lower_directly() {
        assert_eq!(
            value_of("grid-column-start: 1"),
            PropertyValue::GridLine(GridLineValue::Line(1))
        );
        assert_eq!(
            value_of("grid-column-end: span 3"),
            PropertyValue::GridLine(GridLineValue::Span(3))
        );
        assert_eq!(
            value_of("grid-row-start: auto"),
            PropertyValue::GridLine(GridLineValue::Auto)
        );
        assert_eq!(
            standard("grid-row-end: 4").0.property(),
            PropertyId::GridRowEnd
        );
    }

    #[test]
    fn grid_line_longhand_honours_css_wide_keywords() {
        assert_eq!(value_of("grid-column-start: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("grid-row-end: initial"), PropertyValue::Initial);
    }

    #[test]
    fn invalid_grid_line_is_unsupported() {
        // A bare length is not a valid grid line (an integer, `span N`, `auto`, or
        // a `<custom-ident>` name are; a `px` value is not).
        let decls = compile_all("grid-column: 50px");
        assert_eq!(decls.len(), 1);
        assert!(matches!(decls[0], CompiledDeclaration::Unsupported(_)));
        // A zero line index is invalid.
        let (result, _) = compile("grid-column-start: 0");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
        // More than one `/` is not a valid placement.
        let decls = compile_all("grid-column: 1 / 2 / 3");
        assert_eq!(decls.len(), 1);
        assert!(matches!(decls[0], CompiledDeclaration::Unsupported(_)));
    }

    // ---- grid-area shorthand ----

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

    // ---- grid-template-areas ----

    #[test]
    fn grid_template_areas_parses_into_a_template() {
        // `"h h" "s m" "f f"`: three rows of two columns.
        let PropertyValue::GridAreas(template) =
            value_of("grid-template-areas: \"h h\" \"s m\" \"f f\"")
        else {
            panic!("expected a grid-areas value");
        };
        assert_eq!(template.rows(), 3);
        assert_eq!(template.columns(), 2);
        // `h` spans row 1, columns 1-2 → line rect (1..2, 1..3).
        let h = template.area("h").expect("h");
        assert_eq!((h.row_start, h.row_end, h.column_start, h.column_end), (1, 2, 1, 3));
        // `s` is the single cell at row 2, column 1.
        let s = template.area("s").expect("s");
        assert_eq!((s.row_start, s.row_end, s.column_start, s.column_end), (2, 3, 1, 2));
    }

    #[test]
    fn grid_template_areas_null_cell_is_empty() {
        let PropertyValue::GridAreas(template) =
            value_of("grid-template-areas: \"a .\" \". b\"")
        else {
            panic!("expected a grid-areas value");
        };
        assert!(template.area("a").is_some());
        assert!(template.area("b").is_some());
        assert!(template.area(".").is_none());
    }

    #[test]
    fn grid_template_areas_non_rectangular_is_unsupported() {
        // `a` forms an L-shape (a hole in its bounding box) → not a filled rectangle.
        let (result, _) = compile("grid-template-areas: \"a a\" \"a .\"");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
        // A ragged grid (unequal columns) is likewise rejected.
        let (result, _) = compile("grid-template-areas: \"a b\" \"a\"");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    #[test]
    fn grid_template_areas_honours_css_wide_keywords() {
        assert_eq!(value_of("grid-template-areas: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("grid-template-areas: initial"), PropertyValue::Initial);
    }
}
