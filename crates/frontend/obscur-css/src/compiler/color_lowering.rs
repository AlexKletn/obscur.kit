#![doc = include_str!("../../docs/api/compiler/color_lowering.doc/_module.md")]

use super::property_compiler::narrow_f32;
use super::value_lowering::angle_to_degrees;
use crate::parser::ParsedValue;
use obscur_scene::color::convert;
use obscur_style::property::StyleColor;

#[doc = include_str!("../../docs/api/compiler/color_lowering.doc/named_color.md")]
pub(super) fn named_color(keyword: &str) -> Option<StyleColor> {
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

#[doc = include_str!("../../docs/api/compiler/color_lowering.doc/parse_rgb.md")]
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

#[doc = include_str!("../../docs/api/compiler/color_lowering.doc/color_from_function.md")]
pub(super) fn color_from_function(name: &str, args: &[ParsedValue]) -> Option<StyleColor> {
    if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") {
        parse_rgb(args)
    } else if name.eq_ignore_ascii_case("hsl") || name.eq_ignore_ascii_case("hsla") {
        parse_hsl(args)
    } else if name.eq_ignore_ascii_case("oklab") {
        parse_oklab(args)
    } else if name.eq_ignore_ascii_case("oklch") {
        parse_oklch(args)
    } else if name.eq_ignore_ascii_case("lab") {
        parse_lab(args)
    } else if name.eq_ignore_ascii_case("lch") {
        parse_lch(args)
    } else {
        None
    }
}

/// One tokenized argument of a modern-syntax `<color>` function.
#[derive(Clone, Copy)]
enum ColorArg {
    /// A bare `<number>`.
    Number(f64),
    /// A `<percentage>`; stores the numeric part (`50` for `50%`).
    Percent(f64),
    /// An `<angle>` normalized to degrees.
    Angle(f64),
    /// The `none` keyword (a missing/powerless component).
    None,
}

/// Splits a color function's argument list into its color components and an
/// optional alpha, tolerating both the legacy comma syntax and the modern
/// space-plus-`/` syntax (spec CSS Color 4 §4). Commas separate components;
/// a single `/` delimiter introduces the alpha. Returns [`None`] on an
/// unrecognised token or a malformed (double-`/`) list.
fn split_color_args(args: &[ParsedValue]) -> Option<(Vec<ColorArg>, Option<ColorArg>)> {
    let mut comps = Vec::new();
    let mut alpha: Option<ColorArg> = None;
    let mut after_slash = false;
    for arg in args {
        match arg {
            ParsedValue::Comma => {}
            ParsedValue::Delimiter('/') => {
                if after_slash {
                    return None;
                }
                after_slash = true;
            }
            other => {
                let c = color_arg(other)?;
                if after_slash {
                    if alpha.is_some() {
                        return None;
                    }
                    alpha = Some(c);
                } else {
                    comps.push(c);
                }
            }
        }
    }
    Some((comps, alpha))
}

/// Tokenizes one component value into a [`ColorArg`], or [`None`] if it is not a
/// number/percentage/angle/`none`.
fn color_arg(value: &ParsedValue) -> Option<ColorArg> {
    match value {
        ParsedValue::Number(n) => Some(ColorArg::Number(*n)),
        ParsedValue::Percentage(p) => Some(ColorArg::Percent(*p)),
        ParsedValue::Dimension { value, unit } => {
            angle_to_degrees(*value, unit).map(|d| ColorArg::Angle(f64::from(d)))
        }
        ParsedValue::Keyword(k) if k.eq_ignore_ascii_case("none") => Some(ColorArg::None),
        _ => None,
    }
}

/// Reads a `<hue>` component (an angle in degrees). A bare number is degrees; an
/// `<angle>` is already degrees; `none` is a powerless `0` (spec CSS Color 4).
fn read_hue(c: ColorArg) -> Option<f64> {
    match c {
        ColorArg::Number(n) => Some(n),
        ColorArg::Angle(d) => Some(d),
        ColorArg::None => Some(0.0),
        ColorArg::Percent(_) => None,
    }
}

/// Resolves the optional alpha to `0.0..=1.0`: a number is a fraction, a
/// percentage is `p/100`, an absent or `none` alpha resolves to `1.0` / `0.0`
/// respectively (spec CSS Color 4 §4).
fn resolve_alpha(alpha: Option<ColorArg>) -> Option<f64> {
    match alpha {
        None => Some(1.0),
        Some(ColorArg::Number(n)) => Some(n),
        Some(ColorArg::Percent(p)) => Some(p / 100.0),
        Some(ColorArg::None) => Some(0.0),
        Some(ColorArg::Angle(_)) => None,
    }
}

/// Builds a clamped-alpha [`StyleColor`] from already-sRGB channels.
fn make_color([r, g, b]: [f64; 3], a: f64) -> StyleColor {
    StyleColor::rgba(
        narrow_f32(r),
        narrow_f32(g),
        narrow_f32(b),
        narrow_f32(a.clamp(0.0, 1.0)),
    )
}

#[doc = include_str!("../../docs/api/compiler/color_lowering.doc/parse_hsl.md")]
fn parse_hsl(args: &[ParsedValue]) -> Option<StyleColor> {
    let (comps, alpha) = split_color_args(args)?;
    if comps.len() != 3 {
        return None;
    }
    // s/l: percentage or number, both scaled by /100 into 0..1.
    let pct = |c: ColorArg| -> Option<f64> {
        match c {
            ColorArg::Percent(p) | ColorArg::Number(p) => Some(p / 100.0),
            ColorArg::None => Some(0.0),
            ColorArg::Angle(_) => None,
        }
    };
    let h = read_hue(comps[0])?;
    let s = pct(comps[1])?;
    let l = pct(comps[2])?;
    let a = resolve_alpha(alpha)?;
    Some(make_color(convert::hsl_to_srgb(h, s, l), a))
}

/// Reads the Oklab lightness `L`: a number is `0..=1`, a percentage is `p/100`,
/// `none` is `0` (spec CSS Color 4 §9.2).
fn read_ok_lightness(c: ColorArg) -> Option<f64> {
    match c {
        ColorArg::Number(n) => Some(n),
        ColorArg::Percent(p) => Some(p / 100.0),
        ColorArg::None => Some(0.0),
        ColorArg::Angle(_) => None,
    }
}

/// Reads an Oklab `a`/`b` (or Oklch `C`): a number is used directly, a
/// percentage is referenced to `±0.4` (`100% = 0.4`), `none` is `0` (spec CSS
/// Color 4 §9.2/§9.3).
fn read_ok_axis(c: ColorArg) -> Option<f64> {
    match c {
        ColorArg::Number(n) => Some(n),
        ColorArg::Percent(p) => Some(p / 100.0 * 0.4),
        ColorArg::None => Some(0.0),
        ColorArg::Angle(_) => None,
    }
}

/// Parses `oklab()` into a [`StyleColor`] (spec CSS Color 4 §9.2). `L` in
/// `[0,1]`/`%`, signed `a`/`b`, optional `/ <alpha>`.
fn parse_oklab(args: &[ParsedValue]) -> Option<StyleColor> {
    let (comps, alpha) = split_color_args(args)?;
    if comps.len() != 3 {
        return None;
    }
    let l = read_ok_lightness(comps[0])?;
    let a = read_ok_axis(comps[1])?;
    let b = read_ok_axis(comps[2])?;
    let alpha = resolve_alpha(alpha)?;
    Some(make_color(convert::oklab_to_srgb(l, a, b), alpha))
}

/// Parses `oklch()` into a [`StyleColor`] (spec CSS Color 4 §9.3). `L` in
/// `[0,1]`/`%`, `C >= 0`, hue angle, optional `/ <alpha>`.
fn parse_oklch(args: &[ParsedValue]) -> Option<StyleColor> {
    let (comps, alpha) = split_color_args(args)?;
    if comps.len() != 3 {
        return None;
    }
    let l = read_ok_lightness(comps[0])?;
    let c = read_ok_axis(comps[1])?.max(0.0);
    let h = read_hue(comps[2])?;
    let alpha = resolve_alpha(alpha)?;
    Some(make_color(convert::oklch_to_srgb(l, c, h), alpha))
}

/// Reads the Lab lightness `L`: a number is `0..=100`, a percentage is `p`
/// (`100% = 100`), `none` is `0` (spec CSS Color 4 §8.1).
fn read_lab_lightness(c: ColorArg) -> Option<f64> {
    match c {
        ColorArg::Number(n) => Some(n),
        ColorArg::Percent(p) => Some(p),
        ColorArg::None => Some(0.0),
        ColorArg::Angle(_) => None,
    }
}

/// Reads a Lab `a`/`b`: a number is used directly, a percentage is referenced
/// to `±125` (`100% = 125`), `none` is `0` (spec CSS Color 4 §8.1).
fn read_lab_axis(c: ColorArg) -> Option<f64> {
    match c {
        ColorArg::Number(n) => Some(n),
        ColorArg::Percent(p) => Some(p / 100.0 * 125.0),
        ColorArg::None => Some(0.0),
        ColorArg::Angle(_) => None,
    }
}

/// Parses `lab()` into a [`StyleColor`] (spec CSS Color 4 §8.1, D50). `L` in
/// `[0,100]`/`%`, signed `a`/`b`, optional `/ <alpha>`.
fn parse_lab(args: &[ParsedValue]) -> Option<StyleColor> {
    let (comps, alpha) = split_color_args(args)?;
    if comps.len() != 3 {
        return None;
    }
    let l = read_lab_lightness(comps[0])?;
    let a = read_lab_axis(comps[1])?;
    let b = read_lab_axis(comps[2])?;
    let alpha = resolve_alpha(alpha)?;
    Some(make_color(convert::lab_to_srgb(l, a, b), alpha))
}

/// Parses `lch()` into a [`StyleColor`] (spec CSS Color 4 §8.3, D50). `L` in
/// `[0,100]`/`%`, `C >= 0` (`100% = 150`), hue angle, optional `/ <alpha>`.
fn parse_lch(args: &[ParsedValue]) -> Option<StyleColor> {
    let (comps, alpha) = split_color_args(args)?;
    if comps.len() != 3 {
        return None;
    }
    let l = read_lab_lightness(comps[0])?;
    // Lch chroma references 100% to 150.
    let c = match comps[1] {
        ColorArg::Number(n) => n,
        ColorArg::Percent(p) => p / 100.0 * 150.0,
        ColorArg::None => 0.0,
        ColorArg::Angle(_) => return None,
    }
    .max(0.0);
    let h = read_hue(comps[2])?;
    let alpha = resolve_alpha(alpha)?;
    Some(make_color(convert::lch_to_srgb(l, c, h), alpha))
}

/// Parses a 3/4/6/8-digit hex colour (without the leading `#`) into a
/// [`StyleColor`], returning [`None`] on a bad length or non-hex digit.
pub(super) fn parse_hex_color(hex: &str) -> Option<StyleColor> {
    // A `#` token carries an identifier, which admits every code point at or
    // above U+0080 (css-syntax-3 §4.2), so its bytes need not be its
    // characters. Refusing anything but hex digits is what makes the byte
    // count below a digit count, and every byte an ASCII one.
    if hex.is_empty() || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let pair = |high: u8, low: u8| {
        let digits = [high, low];
        let text = std::str::from_utf8(&digits).ok()?;
        u8::from_str_radix(text, 16)
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
            pair(bytes[0], bytes[1])?,
            pair(bytes[2], bytes[3])?,
            pair(bytes[4], bytes[5])?,
            1.0,
        )),
        8 => Some(StyleColor::rgba(
            pair(bytes[0], bytes[1])?,
            pair(bytes[2], bytes[3])?,
            pair(bytes[4], bytes[5])?,
            pair(bytes[6], bytes[7])?,
        )),
        _ => None,
    }
}

/// Reads one color-stop color: a hex hash, a modelled named color, or any CSS
/// Color 4 `<color>` function (`rgb()`/`hsl()`/`oklab()`/`oklch()`/`lab()`/
/// `lch()`); [`None`] for anything else.
pub(super) fn parse_stop_color(value: &ParsedValue) -> Option<StyleColor> {
    match value {
        ParsedValue::Hash(hex) => parse_hex_color(hex),
        ParsedValue::Keyword(keyword) => named_color(keyword),
        ParsedValue::Function { name, args } => color_from_function(name, args),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::property_compiler::tests::{compile, value_of};

    use crate::compiler::CompiledDeclaration;

    use obscur_style::property::{PropertyValue, StyleColor};

    pub(crate) fn color(r: f32, g: f32, b: f32) -> StyleColor {
        StyleColor::rgba(r, g, b, 1.0)
    }

    /// Extracts the [`StyleColor`] a `color:` declaration lowers to.
    fn color_of(text: &str) -> StyleColor {
        match value_of(text) {
            PropertyValue::Color(c) => c,
            other => panic!("expected color, got {other:?}"),
        }
    }

    /// Asserts two colors are equal within `eps` per channel.
    fn color_close(got: StyleColor, want: StyleColor, eps: f32) {
        assert!(
            (got.r - want.r).abs() <= eps
                && (got.g - want.g).abs() <= eps
                && (got.b - want.b).abs() <= eps
                && (got.a - want.a).abs() <= eps,
            "got {got:?} want {want:?} (eps {eps})",
        );
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
    fn lowers_hsl_modern_and_legacy() {
        // Modern space syntax.
        color_close(
            color_of("color: hsl(120 100% 50%)"),
            StyleColor::rgba(0.0, 1.0, 0.0, 1.0),
            1e-2,
        );
        // Legacy comma syntax; `hsla` alias.
        color_close(
            color_of("color: hsl(0, 100%, 50%)"),
            StyleColor::rgba(1.0, 0.0, 0.0, 1.0),
            1e-2,
        );
        // Modern alpha via `/`.
        color_close(
            color_of("color: hsla(240 100% 50% / 0.5)"),
            StyleColor::rgba(0.0, 0.0, 1.0, 0.5),
            1e-2,
        );
    }

    #[test]
    fn lowers_oklab_and_oklch() {
        // oklab(0.627955 0.224863 0.125846) ~= sRGB red.
        color_close(
            color_of("color: oklab(0.627955 0.224863 0.125846)"),
            StyleColor::rgba(1.0, 0.0, 0.0, 1.0),
            1e-2,
        );
        color_close(
            color_of("color: oklab(1 0 0)"),
            StyleColor::rgba(1.0, 1.0, 1.0, 1.0),
            1e-2,
        );
        // oklch(0.62796 0.25768 29.234) ~= red.
        color_close(
            color_of("color: oklch(0.62796 0.25768 29.234)"),
            StyleColor::rgba(1.0, 0.0, 0.0, 1.0),
            1e-2,
        );
        // oklch(0.86644 0.29483 142.5) ~= green, with alpha.
        color_close(
            color_of("color: oklch(0.86644 0.29483 142.5 / 50%)"),
            StyleColor::rgba(0.0, 1.0, 0.0, 0.5),
            1e-2,
        );
    }

    #[test]
    fn lowers_lab_and_lch() {
        color_close(
            color_of("color: lab(100% 0 0)"),
            StyleColor::rgba(1.0, 1.0, 1.0, 1.0),
            1e-2,
        );
        color_close(
            color_of("color: lab(0% 0 0)"),
            StyleColor::rgba(0.0, 0.0, 0.0, 1.0),
            1e-2,
        );
        // A mid reddish lab(); R > G > B and in gamut.
        let mid = color_of("color: lab(50% 40 30)");
        assert!(mid.r > mid.g && mid.g > mid.b, "R>G>B: {mid:?}");
        // lch with hue 0 equals lab with a = C, b = 0.
        color_close(
            color_of("color: lch(50% 50 0)"),
            color_of("color: lab(50% 50 0)"),
            1e-4,
        );
    }

    #[test]
    fn a_hex_color_carrying_a_non_ascii_character_is_not_a_color() {
        // A `#` token's payload is an identifier, and css-syntax-3 §4.2 lets an
        // identifier hold any code point at or above U+0080. `#aébcd` is six
        // BYTES and five characters, so counting bytes would take it for a
        // six-digit colour and then cut the `é` in half reading the first pair.
        let (result, _) = compile("color: #aébcd");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
        // Eight bytes, likewise not eight digits.
        let (result, _) = compile("color: #aébcdef");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    #[test]
    fn bad_hex_length_is_unsupported() {
        // `#12345` is neither 3/4/6/8 digits: the value cannot lower.
        let (result, _) = compile("color: #12345");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }
}
