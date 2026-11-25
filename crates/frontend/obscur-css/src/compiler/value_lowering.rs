#![doc = include_str!("../../docs/api/compiler/value_lowering.doc/_module.md")]

use super::CssSymbolInterner;
use super::calc_lowering::lower_math_function;
use super::color_lowering::{color_from_function, named_color, parse_hex_color};
use super::property_compiler::narrow_f32;
use crate::parser::ParsedValue;
use obscur_identity::InternedSymbol;
use obscur_style::property::{EnvVar, Length, PropertyValue, layout_keyword};

/// Maps a CSS-wide keyword (`inherit` / `initial` / `unset`) to its
/// [`PropertyValue`], case-insensitively; [`None`] for any other keyword.
pub(super) fn css_wide_keyword(keyword: &str) -> Option<PropertyValue> {
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

#[doc = include_str!("../../docs/api/compiler/value_lowering.doc/lower_value.md")]
pub(super) fn lower_value(
    values: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    values.first().and_then(|v| lower_component(v, interner))
}

/// Lowers one component value into a [`PropertyValue`], or [`None`].
pub(super) fn lower_component(
    value: &ParsedValue,
    interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
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

#[doc = include_str!("../../docs/api/compiler/value_lowering.doc/lower_keyword.md")]
fn lower_keyword(keyword: &str, interner: &mut CssSymbolInterner) -> PropertyValue {
    if let Some(wide) = css_wide_keyword(keyword) {
        return wide;
    }
    if keyword.eq_ignore_ascii_case("auto") {
        return PropertyValue::Length(Length::Auto);
    }
    if let Some(sizing) = layout_keyword::sizing_from_name(keyword) {
        return PropertyValue::Keyword(sizing);
    }
    if let Some(color) = named_color(keyword) {
        return PropertyValue::Color(color);
    }
    PropertyValue::Keyword(interner.intern(keyword))
}

#[doc = include_str!("../../docs/api/compiler/value_lowering.doc/dimension_length.md")]
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
    } else if unit.eq_ignore_ascii_case("ch") {
        Some(Length::Ch(n))
    } else if unit.eq_ignore_ascii_case("vw") {
        Some(Length::Vw(n))
    } else if unit.eq_ignore_ascii_case("vh") {
        Some(Length::Vh(n))
    } else if unit.eq_ignore_ascii_case("vmin") {
        Some(Length::Vmin(n))
    } else if unit.eq_ignore_ascii_case("vmax") {
        Some(Length::Vmax(n))
    } else if let Some(per_unit) = absolute_px_per_unit(unit) {
        Some(Length::px(narrow_f32(value * f64::from(per_unit))))
    } else if unit.eq_ignore_ascii_case("cqw") {
        Some(Length::Cqw(n))
    } else if unit.eq_ignore_ascii_case("cqh") {
        Some(Length::Cqh(n))
    } else if unit.eq_ignore_ascii_case("cqi") {
        Some(Length::Cqi(n))
    } else if unit.eq_ignore_ascii_case("cqb") {
        Some(Length::Cqb(n))
    } else if unit.eq_ignore_ascii_case("cqmin") {
        Some(Length::Cqmin(n))
    } else if unit.eq_ignore_ascii_case("cqmax") {
        Some(Length::Cqmax(n))
    } else {
        None
    }
}

#[doc = include_str!("../../docs/api/compiler/value_lowering.doc/absolute_px_per_unit.md")]
fn absolute_px_per_unit(unit: &str) -> Option<f32> {
    /// `1in = 96px`.
    const PX_PER_IN: f32 = 96.0;
    /// `1cm = 96px/2.54`.
    const PX_PER_CM: f32 = PX_PER_IN / 2.54;

    if unit.eq_ignore_ascii_case("cm") {
        Some(PX_PER_CM)
    } else if unit.eq_ignore_ascii_case("mm") {
        Some(PX_PER_CM / 10.0)
    } else if unit.eq_ignore_ascii_case("q") {
        Some(PX_PER_CM / 40.0)
    } else if unit.eq_ignore_ascii_case("in") {
        Some(PX_PER_IN)
    } else if unit.eq_ignore_ascii_case("pc") {
        Some(PX_PER_IN / 6.0)
    } else if unit.eq_ignore_ascii_case("pt") {
        Some(PX_PER_IN / 72.0)
    } else {
        None
    }
}

/// Lowers a dimension into a [`PropertyValue::Length`], via [`dimension_length`].
fn lower_dimension(value: f64, unit: &str) -> Option<PropertyValue> {
    dimension_length(value, unit).map(PropertyValue::Length)
}

#[doc = include_str!("../../docs/api/compiler/value_lowering.doc/lower_function.md")]
fn lower_function(
    name: &str,
    args: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    if name.eq_ignore_ascii_case("var") {
        return var_reference(args);
    }
    if name.eq_ignore_ascii_case("env") {
        return lower_env(args, interner);
    }
    if let Some(color) = color_from_function(name, args) {
        return Some(PropertyValue::Color(color));
    }
    lower_math_function(name, args).map(PropertyValue::Calc)
}

#[doc = include_str!("../../docs/api/compiler/value_lowering.doc/var_reference.md")]
fn var_reference(args: &[ParsedValue]) -> Option<PropertyValue> {
    args.iter().find_map(|arg| match arg {
        // The reference is minted from the NAME, never from this sheet's
        // interner, for the reason `property_compiler` gives where the
        // declaration is minted: a per-source interner numbers from zero, so an
        // interned `--brand` would only ever match a declaration compiled by the
        // same compiler instance — never one from the theme sheet next to it,
        // and never one from a `style` attribute.
        ParsedValue::Keyword(k) if k.starts_with("--") => {
            Some(PropertyValue::CustomRef(InternedSymbol::named(k)))
        }
        _ => None,
    })
}

#[doc = include_str!("../../docs/api/compiler/value_lowering.doc/custom_ref.md")]
pub(super) fn custom_ref(value: &ParsedValue) -> Option<PropertyValue> {
    match value {
        ParsedValue::Function { name, args } if name.eq_ignore_ascii_case("var") => {
            var_reference(args)
        }
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/compiler/value_lowering.doc/sole_custom_ref.md")]
pub(super) fn sole_custom_ref(values: &[ParsedValue]) -> Option<PropertyValue> {
    match values {
        [only] => custom_ref(only),
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/compiler/value_lowering.doc/lower_env.md")]
fn lower_env(args: &[ParsedValue], interner: &mut CssSymbolInterner) -> Option<PropertyValue> {
    let mut components = args.iter().filter(|arg| !matches!(arg, ParsedValue::Comma));
    let name = match components.next()? {
        ParsedValue::Keyword(keyword) => keyword.clone(),
        _ => return None,
    };
    if let Some(var) = EnvVar::from_name(&name) {
        return Some(PropertyValue::Length(Length::Env(var)));
    }
    components
        .next()
        .and_then(|fallback| lower_component(fallback, interner))
}

/// Converts an `<angle>` dimension into degrees on the CSS convention, accepting
/// `deg` / `grad` / `rad` / `turn`; any other unit yields [`None`].
pub(super) fn angle_to_degrees(value: f64, unit: &str) -> Option<f32> {
    let v = narrow_f32(value);
    if unit.eq_ignore_ascii_case("deg") {
        Some(v)
    } else if unit.eq_ignore_ascii_case("grad") {
        // 400 grad = 360 deg.
        Some(v * 0.9)
    } else if unit.eq_ignore_ascii_case("rad") {
        Some(narrow_f32(value.to_degrees()))
    } else if unit.eq_ignore_ascii_case("turn") {
        Some(narrow_f32(value * 360.0))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::property_compiler::tests::{compile, standard, value_of};

    use crate::compiler::CompiledDeclaration;

    use obscur_style::property::{Length, PropertyValue};

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
    fn lowers_visibility_to_reserved_keyword() {
        assert_eq!(
            value_of("visibility: hidden"),
            PropertyValue::Keyword(obscur_style::property::layout_keyword::VISIBILITY_HIDDEN),
        );
    }

    #[test]
    fn lowers_var_to_custom_ref() {
        use obscur_identity::InternedSymbol;

        let (d, _) = standard("color: var(--brand)");
        let PropertyValue::CustomRef(symbol) = d.value().clone() else {
            panic!("expected custom ref");
        };
        // Named rather than interned: the sheet this reference was written in
        // is usually not the sheet the declaration was written in, and a
        // per-source interner would give the two different numbers.
        assert_eq!(symbol, InternedSymbol::named("--brand"));
        assert_ne!(symbol, InternedSymbol::named("--other"));
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
    fn lowers_ch_against_the_measured_zero_advance() {
        use obscur_style::property::LengthContext;
        // A face whose `0` advances 10px, at font-size 16 — deliberately not
        // 0.5em (8), so a `ch` that ignored the measurement would show up here.
        let ctx = LengthContext::new(16.0, 20.0, 800.0, 600.0).with_ch_width(10.0);
        let px = |text: &str| match value_of(text) {
            PropertyValue::Length(length) => length.resolve_px(&ctx),
            other => panic!("expected length, got {other:?}"),
        };
        assert_eq!(px("width: 20ch"), Some(200.0), "twenty `0`-advances wide");
        assert_eq!(px("width: 1CH"), Some(10.0), "units are case-insensitive");
        // Fractional and zero coefficients carry through unremarkably.
        assert_eq!(px("width: 0ch"), Some(0.0));
        assert_eq!(px("width: 2.5ch"), Some(25.0));
    }

    #[test]
    fn ch_composes_inside_calc() {
        use obscur_style::property::LengthContext;
        let ctx = LengthContext::new(16.0, 20.0, 800.0, 600.0).with_ch_width(10.0);
        let PropertyValue::Calc(expr) = value_of("width: calc(20ch + 8px)") else {
            panic!("expected calc");
        };
        // A `ch` is an ordinary term in the expression grammar: 20*10 + 8.
        assert_eq!(expr.resolve(&ctx), Some(208.0));
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

    /// css-values-4 §10.2: `clamp(none, VAL, MAX)` is `min(VAL, MAX)`,
    /// `clamp(MIN, VAL, none)` is `max(MIN, VAL)`, and `clamp(none, VAL, none)`
    /// is just `VAL`. The third case is what makes `none` a keyword rather than
    /// an infinity worth pattern-matching on downstream.
    #[test]
    fn clamp_accepts_none_as_either_bound() {
        use obscur_style::property::LengthContext;
        let ctx = LengthContext::new(16.0, 20.0, 800.0, 600.0);
        let resolved = |text: &str| {
            let PropertyValue::Calc(expr) = value_of(text) else {
                panic!("expected calc for {text}");
            };
            expr.resolve(&ctx)
        };
        assert_eq!(resolved("width: clamp(none, 30px, 20px)"), Some(20.0));
        assert_eq!(resolved("width: clamp(40px, 30px, none)"), Some(40.0));
        assert_eq!(resolved("width: clamp(none, 30px, none)"), Some(30.0));
        // `none` is a whole-argument keyword, not a term inside an expression.
        assert!(matches!(
            compile("width: clamp(none + 1px, 30px, 20px)").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }

    /// css-values-4 §6.2 fixes every absolute unit against the CSS pixel:
    /// `1cm = 96px/2.54`, `1mm = 1/10th of 1cm`, `1Q = 1/40th of 1cm`,
    /// `1in = 96px`, `1pc = 1/6th of 1in`, `1pt = 1/72nd of 1in`. Unit names are
    /// ASCII case-insensitive.
    #[test]
    fn absolute_units_lower_to_their_fixed_pixel_ratios() {
        let cm = 96.0 / 2.54;
        for (text, expected) in [
            ("width: 1cm", cm),
            ("width: 1mm", cm / 10.0),
            ("width: 1Q", cm / 40.0),
            ("width: 1q", cm / 40.0),
            ("width: 1in", 96.0),
            ("width: 1IN", 96.0),
            ("width: 1pc", 16.0),
            ("width: 12pt", 16.0),
        ] {
            let PropertyValue::Length(Length::Px(px)) = value_of(text) else {
                panic!("expected an absolute px length for {text}");
            };
            assert!(
                (px.0 - expected).abs() < 1e-3,
                "{text} lowered to {px:?}, expected {expected}"
            );
        }
    }

    #[test]
    fn container_units_lower_to_their_own_terms() {
        for (text, expected) in [
            ("width: 5cqw", Length::Cqw(5.0)),
            ("width: 5cqh", Length::Cqh(5.0)),
            ("width: 5cqi", Length::Cqi(5.0)),
            ("width: 5cqb", Length::Cqb(5.0)),
            ("width: 5cqmin", Length::Cqmin(5.0)),
            ("width: 5CQMAX", Length::Cqmax(5.0)),
        ] {
            assert_eq!(value_of(text), PropertyValue::Length(expected), "{text}");
        }
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
}
