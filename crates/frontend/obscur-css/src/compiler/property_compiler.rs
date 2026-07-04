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

use obscur_style::cascade::{CascadeOrigin, Declaration, ImportantFlag};
use obscur_style::property::{CustomProperty, Length, PropertyId, PropertyValue, StyleColor};
use obscur_style::selector::{SelectorId, SelectorSpecificity};

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
        match lower_value(&decl.value, interner) {
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
}

/// Resolves a CSS property name to its [`PropertyId`], case-insensitively.
///
/// Returns [`None`] for any name outside the seven modelled longhands (custom
/// properties are handled separately by their `--` prefix).
#[must_use]
pub(crate) fn property_id_from_name(name: &str) -> Option<PropertyId> {
    const ALL: [PropertyId; 7] = [
        PropertyId::Color,
        PropertyId::BackgroundColor,
        PropertyId::Opacity,
        PropertyId::Width,
        PropertyId::Height,
        PropertyId::Display,
        PropertyId::Visibility,
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

/// Lowers a component-value list into a single [`PropertyValue`].
///
/// Only the first component is inspected: the modelled properties are all
/// single-valued, and multi-component shorthands are out of the domain's value
/// model. Returns [`None`] when the component cannot be lowered.
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
    if keyword.eq_ignore_ascii_case("inherit") {
        return PropertyValue::Inherit;
    }
    if keyword.eq_ignore_ascii_case("initial") {
        return PropertyValue::Initial;
    }
    if keyword.eq_ignore_ascii_case("unset") {
        return PropertyValue::Unset;
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

/// Lowers a dimension, accepting `px` (absolute) and `%` (percentage); any other
/// unit is outside the domain's [`Length`] model and yields [`None`].
fn lower_dimension(value: f64, unit: &str) -> Option<PropertyValue> {
    if unit.eq_ignore_ascii_case("px") {
        Some(PropertyValue::Length(Length::px(narrow_f32(value))))
    } else if unit == "%" {
        Some(PropertyValue::Length(Length::Percent(narrow_f32(value))))
    } else {
        None
    }
}

/// Lowers a function value: `var(--name)` to a [`PropertyValue::CustomRef`] and
/// `rgb()`/`rgba()` to a colour; other functions (for example `calc()`) are not
/// modelled and yield [`None`].
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
    None
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
        let (result, _) = compile("margin: 1px");
        let CompiledDeclaration::Unsupported(feature) = result else {
            panic!("expected unsupported");
        };
        assert_eq!(feature.kind(), UnsupportedFeatureKind::Property);
        assert_eq!(feature.name(), "margin");
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
        let (d, interner) = standard("display: flex");
        let PropertyValue::Keyword(symbol) = d.value().clone() else {
            panic!("expected keyword");
        };
        assert_eq!(interner.resolve(symbol), Some("flex"));
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
    fn unsupported_unit_reports_unit_kind() {
        let (result, _) = compile("width: 3em");
        let CompiledDeclaration::Unsupported(feature) = result else {
            panic!("expected unsupported");
        };
        assert_eq!(feature.kind(), UnsupportedFeatureKind::Unit);
    }

    #[test]
    fn unsupported_function_reports_function_kind() {
        let (result, _) = compile("width: calc(100% - 20px)");
        let CompiledDeclaration::Unsupported(feature) = result else {
            panic!("expected unsupported");
        };
        assert_eq!(feature.kind(), UnsupportedFeatureKind::Function);
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
}
