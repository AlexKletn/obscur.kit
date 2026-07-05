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
    layout_keyword, CustomProperty, GridLineValue, GridTrack, Length, PropertyId, PropertyValue,
    StyleColor,
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

/// Lowers one `<grid-line>` edge from a component group: `auto`, a non-zero
/// integer line index (`3`, `-1`), or `span N` (`N` a positive integer, in either
/// order `span 2` / `2 span`). Returns [`None`] for anything else.
fn grid_line_edge(group: &[ParsedValue]) -> Option<GridLineValue> {
    match group {
        [ParsedValue::Keyword(keyword)] if keyword.eq_ignore_ascii_case("auto") => {
            Some(GridLineValue::Auto)
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
    const ALL: [PropertyId; 39] = [
        PropertyId::Color,
        PropertyId::BackgroundColor,
        PropertyId::Opacity,
        PropertyId::Width,
        PropertyId::Height,
        PropertyId::Display,
        PropertyId::Visibility,
        PropertyId::MarginTop,
        PropertyId::MarginRight,
        PropertyId::MarginBottom,
        PropertyId::MarginLeft,
        PropertyId::PaddingTop,
        PropertyId::PaddingRight,
        PropertyId::PaddingBottom,
        PropertyId::PaddingLeft,
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
        PropertyId::AlignItems,
        PropertyId::GridTemplateColumns,
        PropertyId::GridTemplateRows,
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
    lower_value(values, interner)
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
        PropertyId::FlexDirection => Some(layout_keyword::flex_direction_from_name),
        PropertyId::JustifyContent => Some(layout_keyword::justify_content_from_name),
        PropertyId::AlignItems => Some(layout_keyword::align_items_from_name),
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
    let tracks = tracks_from_components(values)?;
    if tracks.is_empty() {
        return None;
    }
    Some(PropertyValue::grid_tracks(tracks))
}

/// Parses a space-separated component list into a flat [`GridTrack`] list,
/// expanding any `repeat(…)` component in place.
fn tracks_from_components(values: &[ParsedValue]) -> Option<Vec<GridTrack>> {
    let mut out = Vec::new();
    for value in values {
        match value {
            // A comma has no meaning inside a track list; reject rather than drop.
            ParsedValue::Comma => return None,
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

/// Expands `repeat(N, <tracks>)` into `N` flat copies of its inner track list.
///
/// `N` is a positive integer (clamped to [`MAX_REPEAT`]); the `auto-fill` /
/// `auto-fit` keywords cannot be resolved without the container width, so they are
/// treated as a single repetition (a documented limitation). Returns [`None`] on a
/// missing count, an empty inner list, or an unparseable inner track.
fn expand_repeat(args: &[ParsedValue]) -> Option<Vec<GridTrack>> {
    let comma = args.iter().position(|v| matches!(v, ParsedValue::Comma))?;
    let (count_part, rest) = args.split_at(comma);
    let inner = tracks_from_components(&rest[1..])?;
    if inner.is_empty() {
        return None;
    }
    let count = repeat_count(count_part)?;
    let mut out = Vec::with_capacity(inner.len().saturating_mul(count));
    for _ in 0..count {
        out.extend(inner.iter().cloned());
    }
    Some(out)
}

/// Resolves the count component of a `repeat(N, …)`: a positive integer clamped to
/// [`MAX_REPEAT`], or `1` for the `auto-fill` / `auto-fit` keywords (unsupported,
/// treated as a single repetition). [`None`] for any other count.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn repeat_count(part: &[ParsedValue]) -> Option<usize> {
    match single_component(part)? {
        ParsedValue::Number(n) if *n >= 1.0 => Some((*n as usize).min(MAX_REPEAT)),
        ParsedValue::Keyword(keyword)
            if keyword.eq_ignore_ascii_case("auto-fill")
                || keyword.eq_ignore_ascii_case("auto-fit") =>
        {
            Some(1)
        }
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
        ParsedValue::String(_) | ParsedValue::Comma | ParsedValue::Delimiter(_) => None,
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
    if keyword.eq_ignore_ascii_case("transparent") {
        return PropertyValue::Color(StyleColor::TRANSPARENT);
    }
    if keyword.eq_ignore_ascii_case("black") {
        return PropertyValue::Color(StyleColor::BLACK);
    }
    PropertyValue::Keyword(interner.intern(keyword))
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
            PropertyValue::GridTracks(list) => list.into_vec(),
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
    fn auto_fill_repeat_is_a_single_repetition() {
        // `auto-fill` cannot be resolved without a container width, so it is
        // treated as one repetition (documented limitation).
        assert_eq!(
            tracks("grid-template-columns: repeat(auto-fill, 100px)"),
            vec![GridTrack::Fixed(Length::px(100.0))]
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
                (decl.property(), *edge)
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
        // A bare colour is not a valid grid line.
        let decls = compile_all("grid-column: red");
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
}
