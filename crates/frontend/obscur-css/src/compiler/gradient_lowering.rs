#![doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/_module.md")]

mod gradient_direction;
mod repeat_pos;

use gradient_direction::{GradientDirection, parse_gradient_direction};
use repeat_pos::RepeatPos;

use super::color_lowering::parse_stop_color;
use super::property_compiler::narrow_f32;
use super::transform_lowering::{OriginAxis, origin_component};
use super::value_lowering::{angle_to_degrees, dimension_length};
use crate::parser::ParsedValue;
use obscur_style::property::{
    ConicGradient, GradientStop, Length, LinearGradient, RadialGradient, RadialShape, RadialSize,
    StyleColor,
};

#[doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/lower_linear_gradient.md")]
pub(super) fn lower_linear_gradient(args: &[ParsedValue]) -> Option<LinearGradient> {
    let parts: Vec<&[ParsedValue]> = args.split(|v| matches!(v, ParsedValue::Comma)).collect();
    if parts.is_empty() {
        return None;
    }
    // The optional leading direction; when the first part is not a direction it is
    // a color stop and the direction defaults to `to bottom` (180°).
    let (direction, stop_parts) = match parse_gradient_direction(parts[0]) {
        Some(direction) => (direction, &parts[1..]),
        None => (GradientDirection::Angle(180.0), &parts[..]),
    };
    if stop_parts.len() < 2 {
        return None;
    }
    let mut stops: Vec<GradientStop> = Vec::with_capacity(stop_parts.len());
    for part in stop_parts {
        stops.push(parse_color_stop(part)?);
    }
    resolve_stop_positions(&mut stops);
    Some(direction.into_linear(stops))
}

#[doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/lower_repeating_linear_gradient.md")]
pub(super) fn lower_repeating_linear_gradient(args: &[ParsedValue]) -> Option<LinearGradient> {
    let parts: Vec<&[ParsedValue]> = args.split(|v| matches!(v, ParsedValue::Comma)).collect();
    if parts.is_empty() {
        return None;
    }
    let (direction, stop_parts) = match parse_gradient_direction(parts[0]) {
        Some(direction) => (direction, &parts[1..]),
        None => (GradientDirection::Angle(180.0), &parts[..]),
    };
    if stop_parts.len() < 2 {
        return None;
    }
    let gradient = if let Some((stops, period)) = normalized_linear_repeat(stop_parts) {
        LinearGradient::new_repeating(direction.nominal_angle(), stops, period)
    } else {
        let (stops, positions) = positioned_repeat_stops(stop_parts)?;
        LinearGradient::new(direction.nominal_angle(), stops).with_repeat_positions(positions)
    };
    Some(match direction {
        GradientDirection::Corner(corner, _) => gradient.with_corner(corner),
        GradientDirection::Angle(_) => gradient,
    })
}

/// Retains the normalized representation for origin-aligned repeats with one unit and a known positive period.
fn normalized_linear_repeat(stop_parts: &[&[ParsedValue]]) -> Option<(Vec<GradientStop>, Length)> {
    // Flatten stops, expanding the two-position hard-band form into two stops.
    let mut colored: Vec<(StyleColor, Option<RepeatPos>)> = Vec::with_capacity(stop_parts.len());
    for part in stop_parts {
        parse_repeating_color_stop(part, &mut colored)?;
    }
    if colored.len() < 2 {
        return None;
    }
    if colored[0].1.is_some_and(|position| position.value() != 0.0) {
        return None;
    }
    // The period unit: taken from the first positioned stop; every positioned
    // stop must agree (no mixed px/percent).
    let unit_is_px = colored
        .iter()
        .find_map(|(_, pos)| pos.map(RepeatPos::is_px))?;
    if colored
        .iter()
        .any(|(_, pos)| pos.is_some_and(|p| p.is_px() != unit_is_px))
    {
        return None;
    }
    // Resolve every stop's magnitude in the chosen unit: the first defaults to 0;
    // runs of unpositioned stops are spread evenly between positioned neighbours;
    // the last must be positioned (it defines the period).
    let magnitudes = resolve_repeating_positions(&colored)?;
    let period = *magnitudes.last()?;
    if period <= 0.0 {
        return None;
    }
    let stops: Vec<GradientStop> = colored
        .iter()
        .zip(&magnitudes)
        .map(|((color, _), mag)| GradientStop::new(*color, mag / period))
        .collect();
    let period_len = if unit_is_px {
        Length::px(period)
    } else {
        Length::Percent(period)
    };
    Some((stops, period_len))
}

/// Parses one `repeating-linear-gradient` color stop into `out`, supporting the
/// `<color>`, `<color> <pos>`, and hard-band `<color> <pos> <pos>` forms. The
/// hard-band form pushes the color twice (once at each position). Returns [`None`]
/// for an unparseable color or position.
fn parse_repeating_color_stop(
    part: &[ParsedValue],
    out: &mut Vec<(StyleColor, Option<RepeatPos>)>,
) -> Option<()> {
    match part {
        [color] => out.push((parse_stop_color(color)?, None)),
        [color, pos] => out.push((parse_stop_color(color)?, Some(repeat_stop_position(pos)?))),
        [color, p1, p2] => {
            let color = parse_stop_color(color)?;
            out.push((color, Some(repeat_stop_position(p1)?)));
            out.push((color, Some(repeat_stop_position(p2)?)));
        }
        _ => return None,
    }
    Some(())
}

/// Reads a repeating-gradient stop position: a `<percentage>` (kept as its
/// `0..=100` value), an absolute `px` `<length>`, or a bare `0` (a unitless zero
/// offset). Any other length unit or shape yields [`None`].
fn repeat_stop_position(value: &ParsedValue) -> Option<RepeatPos> {
    match value {
        ParsedValue::Percentage(p) => Some(RepeatPos::Pct(narrow_f32(*p))),
        ParsedValue::Dimension { value, unit } if unit.eq_ignore_ascii_case("px") => {
            Some(RepeatPos::Px(narrow_f32(*value)))
        }
        ParsedValue::Number(n) if *n == 0.0 => Some(RepeatPos::Px(0.0)),
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/resolve_repeating_positions.md")]
fn resolve_repeating_positions(colored: &[(StyleColor, Option<RepeatPos>)]) -> Option<Vec<f32>> {
    let n = colored.len();
    let mut mags: Vec<Option<f32>> = colored
        .iter()
        .map(|(_, p)| p.map(RepeatPos::value))
        .collect();
    if mags[0].is_none() {
        mags[0] = Some(0.0);
    }
    // The last stop must be positioned — it defines the period.
    mags[n - 1]?;
    let mut highest_magnitude_so_far = 0.0_f32;
    for m in mags.iter_mut().flatten() {
        *m = m.max(highest_magnitude_so_far);
        highest_magnitude_so_far = *m;
    }
    // Fill runs of unpositioned stops by even interpolation between the known
    // magnitudes bounding the run (both endpoints are known).
    let mut i = 0;
    while i < n {
        if mags[i].is_some() {
            i += 1;
            continue;
        }
        let start = i - 1;
        let prev = mags[start].unwrap_or(0.0);
        let mut j = i;
        while j < n && mags[j].is_none() {
            j += 1;
        }
        let next = mags[j].unwrap_or(prev);
        #[allow(clippy::cast_precision_loss)]
        let span = (j - start) as f32;
        for (offset, k) in (i..j).enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let t = (offset + 1) as f32 / span;
            mags[k] = Some(prev + (next - prev) * t);
        }
        i = j;
    }
    Some(mags.into_iter().map(|m| m.unwrap_or(0.0)).collect())
}

#[doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/lower_radial_gradient.md")]
pub(super) fn lower_radial_gradient(
    args: &[ParsedValue],
    repeating: bool,
) -> Option<RadialGradient> {
    let parts: Vec<&[ParsedValue]> = args.split(|v| matches!(v, ParsedValue::Comma)).collect();
    if parts.is_empty() {
        return None;
    }
    let center = Length::Percent(50.0);
    let (shape, size, cx, cy, stop_parts) = match parse_radial_prelude(parts[0]) {
        Some((shape, size, cx, cy)) => (shape, size, cx, cy, &parts[1..]),
        None => (
            RadialShape::default(),
            RadialSize::default(),
            center,
            center,
            &parts[..],
        ),
    };
    if stop_parts.len() < 2 {
        return None;
    }
    if repeating {
        let (stops, positions) = positioned_repeat_stops(stop_parts)?;
        return Some(
            RadialGradient::new(shape, size, cx, cy, stops).with_repeat_positions(positions),
        );
    }
    let mut stops: Vec<GradientStop> = Vec::with_capacity(stop_parts.len());
    for part in stop_parts {
        stops.push(parse_color_stop(part)?);
    }
    resolve_stop_positions(&mut stops);
    Some(RadialGradient::new(shape, size, cx, cy, stops))
}

/// Parses stop distances for a repeat whose gradient-line length is resolved later.
fn positioned_repeat_stops(
    parts: &[&[ParsedValue]],
) -> Option<(Vec<GradientStop>, Box<[Option<Length>]>)> {
    let mut stops = Vec::new();
    let mut positions = Vec::new();
    let position = |value: &ParsedValue| match value {
        ParsedValue::Dimension { value, unit } => dimension_length(*value, unit),
        ParsedValue::Percentage(value) => Some(Length::Percent(narrow_f32(*value))),
        ParsedValue::Number(value) if *value == 0.0 => Some(Length::px(0.0)),
        _ => None,
    };
    for part in parts {
        let (color, offsets) = part.split_first()?;
        let color = parse_stop_color(color)?;
        if offsets.is_empty() {
            stops.push(GradientStop::auto(color));
            positions.push(None);
        } else if offsets.len() <= 2 {
            for offset in offsets {
                stops.push(GradientStop::auto(color));
                positions.push(Some(position(offset)?));
            }
        } else {
            return None;
        }
    }
    Some((stops, positions.into_boxed_slice()))
}

#[doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/parse_radial_prelude.md")]
fn parse_radial_prelude(part: &[ParsedValue]) -> Option<(RadialShape, RadialSize, Length, Length)> {
    let at_index = part
        .iter()
        .position(|v| matches!(v, ParsedValue::Keyword(k) if k.eq_ignore_ascii_case("at")));
    let (dims, position): (&[ParsedValue], Option<&[ParsedValue]>) = match at_index {
        Some(i) => (&part[..i], Some(&part[i + 1..])),
        None => (part, None),
    };
    let mut shape = None;
    let mut extent = None;
    let mut radii = [None; 2];
    let mut count = 0;
    for token in dims {
        let radius = match token {
            ParsedValue::Keyword(keyword) => {
                let value = match keyword.to_ascii_lowercase().as_str() {
                    "circle" if shape.is_none() => {
                        shape = Some(RadialShape::Circle);
                        continue;
                    }
                    "ellipse" if shape.is_none() => {
                        shape = Some(RadialShape::Ellipse);
                        continue;
                    }
                    "closest-side" => RadialSize::ClosestSide,
                    "closest-corner" => RadialSize::ClosestCorner,
                    "farthest-side" => RadialSize::FarthestSide,
                    "farthest-corner" => RadialSize::FarthestCorner,
                    _ => return None,
                };
                if extent.replace(value).is_some() {
                    return None;
                }
                continue;
            }
            ParsedValue::Dimension { value, unit } if *value >= 0.0 => {
                dimension_length(*value, unit)?
            }
            ParsedValue::Percentage(p) if *p >= 0.0 => Length::Percent(narrow_f32(*p)),
            ParsedValue::Number(n) if *n == 0.0 => Length::px(0.0),
            _ => return None,
        };
        *radii.get_mut(count)? = Some(radius);
        count += 1;
    }
    if dims.is_empty() && position.is_none() {
        return None;
    }
    let (shape, size) = match (extent, radii) {
        (extent, [None, None]) => (shape.unwrap_or_default(), extent.unwrap_or_default()),
        (None, [Some(radius), None]) => (
            shape.unwrap_or(if matches!(radius, Length::Percent(_)) {
                RadialShape::Ellipse
            } else {
                RadialShape::Circle
            }),
            RadialSize::Length(radius),
        ),
        (None, [Some(x), Some(y)]) if shape != Some(RadialShape::Circle) => {
            (RadialShape::Ellipse, RadialSize::Ellipse(x, y))
        }
        _ => return None,
    };
    let center = Length::Percent(50.0);
    let (cx, cy) = match position {
        Some(tokens) => parse_at_position(tokens)?,
        None => (center, center),
    };
    Some((shape, size, cx, cy))
}

#[doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/lower_conic_gradient.md")]
pub(super) fn lower_conic_gradient(args: &[ParsedValue]) -> Option<ConicGradient> {
    let parts: Vec<&[ParsedValue]> = args.split(|v| matches!(v, ParsedValue::Comma)).collect();
    if parts.is_empty() {
        return None;
    }
    let center = Length::Percent(50.0);
    let (angle, cx, cy, stop_parts) = match parse_conic_prelude(parts[0]) {
        Some((angle, cx, cy)) => (angle, cx, cy, &parts[1..]),
        None => (0.0, center, center, &parts[..]),
    };
    if stop_parts.len() < 2 {
        return None;
    }
    let mut stops: Vec<GradientStop> = Vec::with_capacity(stop_parts.len());
    for part in stop_parts {
        stops.push(parse_conic_color_stop(part)?);
    }
    resolve_stop_positions(&mut stops);
    Some(ConicGradient::new(angle, cx, cy, stops))
}

/// Parses the optional leading prelude of a `conic-gradient`: an optional
/// `from <angle>` followed by an optional `at <position>`. Returns [`None`] when
/// the part carries neither `from` nor `at` (so it is treated as the first color
/// stop) or when it is malformed (leftover tokens, bad angle/position).
fn parse_conic_prelude(part: &[ParsedValue]) -> Option<(f32, Length, Length)> {
    let center = Length::Percent(50.0);
    let mut angle = 0.0_f32;
    let mut saw_any = false;
    let mut idx = 0;
    // Optional `from <angle>`.
    if let Some(ParsedValue::Keyword(keyword)) = part.first()
        && keyword.eq_ignore_ascii_case("from")
    {
        angle = match part.get(1)? {
            ParsedValue::Dimension { value, unit } => angle_to_degrees(*value, unit)?,
            ParsedValue::Number(n) if *n == 0.0 => 0.0,
            _ => return None,
        };
        idx = 2;
        saw_any = true;
    }
    // Optional `at <position>` (consumes the remainder).
    let (cx, cy) = if let Some(ParsedValue::Keyword(keyword)) = part.get(idx)
        && keyword.eq_ignore_ascii_case("at")
    {
        saw_any = true;
        parse_at_position(&part[idx + 1..])?
    } else if idx == part.len() {
        (center, center)
    } else {
        // Leftover tokens with no `at`: a malformed prelude.
        return None;
    };
    if !saw_any {
        return None;
    }
    Some((angle, cx, cy))
}

#[doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/parse_at_position.md")]
fn parse_at_position(tokens: &[ParsedValue]) -> Option<(Length, Length)> {
    let center = Length::Percent(50.0);
    match tokens {
        [a] => {
            let (len, axis) = origin_component(a)?;
            Some(match axis {
                OriginAxis::Y => (center, len),
                _ => (len, center),
            })
        }
        [a, b] => {
            let ac = origin_component(a)?;
            let bc = origin_component(b)?;
            let (x, y) = if ac.1 == OriginAxis::Y || bc.1 == OriginAxis::X {
                (bc.0, ac.0)
            } else {
                (ac.0, bc.0)
            };
            Some((x, y))
        }
        _ => None,
    }
}

/// Parses one `<color> [<percentage>]?` color stop from a comma-delimited part: a
/// lone color (auto-positioned) or a color followed by a `<percentage>` position.
/// Returns [`None`] for an unparseable color or an extra component.
fn parse_color_stop(part: &[ParsedValue]) -> Option<GradientStop> {
    match part {
        [color] => Some(GradientStop::auto(parse_stop_color(color)?)),
        [color, position] => {
            let color = parse_stop_color(color)?;
            let position = stop_position(position)?;
            Some(GradientStop::new(color, position))
        }
        _ => None,
    }
}

/// Reads a color-stop position as a normalized `0.0..=1.0` value from a
/// `<percentage>`; other value shapes (lengths, bare numbers) are out of scope and
/// yield [`None`].
fn stop_position(value: &ParsedValue) -> Option<f32> {
    match value {
        ParsedValue::Percentage(p) => Some(narrow_f32(*p) / 100.0),
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/parse_conic_color_stop.md")]
fn parse_conic_color_stop(part: &[ParsedValue]) -> Option<GradientStop> {
    match part {
        [color] => Some(GradientStop::auto(parse_stop_color(color)?)),
        [color, position] => {
            let color = parse_stop_color(color)?;
            let position = conic_stop_position(position)?;
            Some(GradientStop::new(color, position))
        }
        _ => None,
    }
}

/// Reads a conic-gradient stop's used position as a normalized `0.0..=1.0` fraction
/// of a full turn: a `<percentage>` divides by 100, an `<angle>` divides by 360deg,
/// and a bare `0` is the zero angle. Any other shape is out of grammar ([`None`]).
fn conic_stop_position(value: &ParsedValue) -> Option<f32> {
    match value {
        ParsedValue::Percentage(p) => Some(narrow_f32(*p) / 100.0),
        ParsedValue::Dimension { value, unit } => {
            angle_to_degrees(*value, unit).map(|deg| deg / 360.0)
        }
        ParsedValue::Number(n) if *n == 0.0 => Some(0.0),
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/compiler/gradient_lowering.doc/resolve_stop_positions.md")]
fn resolve_stop_positions(stops: &mut [GradientStop]) {
    let n = stops.len();
    if n == 0 {
        return;
    }
    if stops[0].position.is_none() {
        stops[0].position = Some(0.0);
    }
    if stops[n - 1].position.is_none() {
        stops[n - 1].position = Some(1.0);
    }
    let mut highest_position_so_far = 0.0_f32;
    for stop in stops.iter_mut() {
        if let Some(p) = stop.position {
            let clamped = p.max(highest_position_so_far);
            stop.position = Some(clamped);
            highest_position_so_far = clamped;
        }
    }
    // Fill runs of unpositioned stops by even interpolation between the known
    // positions bounding the run. Both endpoints are known (set above), so every
    // run is bounded on both sides.
    let mut i = 0;
    while i < n {
        if stops[i].position.is_some() {
            i += 1;
            continue;
        }
        let start = i - 1;
        let prev = stops[start].position.unwrap_or(0.0);
        let mut j = i;
        while j < n && stops[j].position.is_none() {
            j += 1;
        }
        let next = stops[j].position.unwrap_or(1.0);
        #[allow(clippy::cast_precision_loss)]
        let span = (j - start) as f32;
        for (offset, k) in (i..j).enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let t = (offset + 1) as f32 / span;
            stops[k].position = Some(prev + (next - prev) * t);
        }
        i = j;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::color_lowering::tests::color;
    use super::super::property_compiler::tests::{compile, value_of};

    use crate::compiler::CompiledDeclaration;

    use obscur_style::property::{
        ConicGradient, Corner, Length, LinearGradient, PropertyValue, RadialGradient, RadialShape,
        RadialSize, StyleColor,
    };

    /// Extracts the lowered [`LinearGradient`] from a `background-image`
    /// declaration.
    pub(crate) fn gradient_of(text: &str) -> LinearGradient {
        match value_of(text) {
            PropertyValue::LinearGradient(g) => g,
            other => panic!("expected a linear gradient, got {other:?}"),
        }
    }

    /// Extracts the lowered [`RadialGradient`] from a `background-image`
    /// declaration.
    fn radial_of(text: &str) -> RadialGradient {
        match value_of(text) {
            PropertyValue::RadialGradient(g) => g,
            other => panic!("expected a radial gradient, got {other:?}"),
        }
    }

    /// Extracts the lowered [`ConicGradient`] from a `background-image`
    /// declaration.
    fn conic_of(text: &str) -> ConicGradient {
        match value_of(text) {
            PropertyValue::ConicGradient(g) => g,
            other => panic!("expected a conic gradient, got {other:?}"),
        }
    }

    #[test]
    fn linear_gradient_angle_and_two_stops() {
        // `linear-gradient(90deg, red, blue)` → angle 90, stops [red@0, blue@1].
        let g = gradient_of("background-image: linear-gradient(90deg, red, blue)");
        assert_eq!(g.angle_deg(), 90.0);
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].color(), color(1.0, 0.0, 0.0));
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].color(), color(0.0, 0.0, 1.0));
        assert_eq!(g.stops()[1].position(), Some(1.0));
    }

    #[test]
    fn linear_gradient_is_non_repeating_with_no_period() {
        let g = gradient_of("background-image: linear-gradient(90deg, red, blue)");
        assert!(!g.repeating());
        assert_eq!(g.period(), None);
    }

    #[test]
    fn repeating_linear_gradient_px_hard_bands() {
        // `repeating-linear-gradient(90deg, #000 0 4px, #fff 4px 8px)` = an 8px
        // period of two hard bands (black 0–4px, white 4–8px), normalized over
        // the period to black@0, black@0.5, white@0.5, white@1.
        let g = gradient_of(
            "background-image: repeating-linear-gradient(90deg, #000 0 4px, #fff 4px 8px)",
        );
        assert_eq!(g.angle_deg(), 90.0);
        assert!(g.repeating());
        assert_eq!(g.period(), Some(Length::px(8.0)));
        assert_eq!(g.stops().len(), 4);
        assert_eq!(g.stops()[0].color(), StyleColor::BLACK);
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].color(), StyleColor::BLACK);
        assert_eq!(g.stops()[1].position(), Some(0.5));
        assert_eq!(g.stops()[2].color(), color(1.0, 1.0, 1.0));
        assert_eq!(g.stops()[2].position(), Some(0.5));
        assert_eq!(g.stops()[3].color(), color(1.0, 1.0, 1.0));
        assert_eq!(g.stops()[3].position(), Some(1.0));
    }

    #[test]
    fn repeating_linear_gradient_percent_period() {
        // A `%` period: `red 0%, blue 25%` → 25%-of-line period, stops normalized
        // to red@0, blue@1.
        let g = gradient_of("background-image: repeating-linear-gradient(0deg, red 0%, blue 25%)");
        assert!(g.repeating());
        assert_eq!(g.period(), Some(Length::Percent(25.0)));
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].position(), Some(1.0));
    }

    #[test]
    fn repeating_linear_gradient_default_direction() {
        // No leading direction → defaults to `to bottom` (180°), still repeating.
        let g =
            gradient_of("background-image: repeating-linear-gradient(#000 0 6px, #fff 6px 12px)");
        assert_eq!(g.angle_deg(), 180.0);
        assert!(g.repeating());
        assert_eq!(g.period(), Some(Length::px(12.0)));
    }

    #[test]
    fn linear_gradient_to_corner_carries_symbolic_corner() {
        // `to <corner>` is carried symbolically (aspect-ratio-aware at paint time),
        // not folded to a fixed 45° angle.
        for (css, corner) in [
            ("to top right", Corner::TopRight),
            ("to bottom right", Corner::BottomRight),
            ("to bottom left", Corner::BottomLeft),
            ("to top left", Corner::TopLeft),
            ("to right top", Corner::TopRight),
        ] {
            let g = gradient_of(&format!(
                "background-image: linear-gradient({css}, #000, #fff)"
            ));
            assert_eq!(g.corner(), Some(corner), "corner for `{css}`");
        }
        // A plain side (or angle) carries no corner.
        let side = gradient_of("background-image: linear-gradient(to right, #000, #fff)");
        assert_eq!(side.corner(), None);
        assert_eq!(side.angle_deg(), 90.0);
    }

    #[test]
    fn repeating_linear_gradient_to_corner_carries_corner() {
        let g = gradient_of(
            "background-image: repeating-linear-gradient(to top right, #000 0 6px, #fff 6px 12px)",
        );
        assert_eq!(g.corner(), Some(Corner::TopRight));
        assert!(g.repeating());
    }

    #[test]
    fn linear_gradient_to_right_with_explicit_positions() {
        // `to right` = 90deg; explicit 0% / 100% positions are honored.
        let g = gradient_of("background-image: linear-gradient(to right, #000 0%, #fff 100%)");
        assert_eq!(g.angle_deg(), 90.0);
        assert_eq!(g.stops()[0].color(), StyleColor::BLACK);
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].color(), color(1.0, 1.0, 1.0));
        assert_eq!(g.stops()[1].position(), Some(1.0));
    }

    #[test]
    fn linear_gradient_three_stops_with_mid_position() {
        // A middle stop at 30%; the endpoints auto-distribute to 0 / 1.
        let g = gradient_of("background-image: linear-gradient(red, lime 30%, blue)");
        assert_eq!(g.stops().len(), 3);
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].position(), Some(0.3));
        assert_eq!(g.stops()[2].position(), Some(1.0));
    }

    #[test]
    fn linear_gradient_default_direction_is_to_bottom() {
        // No angle / `to <side>` → the direction defaults to `to bottom` (180°),
        // and the two unpositioned stops distribute to 0 / 1.
        let g = gradient_of("background-image: linear-gradient(red, blue)");
        assert_eq!(g.angle_deg(), 180.0);
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].position(), Some(1.0));
    }

    #[test]
    fn linear_gradient_auto_distributes_interior_stops() {
        // Four unpositioned stops distribute evenly to 0 / 1/3 / 2/3 / 1.
        let g = gradient_of("background-image: linear-gradient(red, lime, blue, black)");
        let positions: Vec<f32> = g.stops().iter().map(|s| s.position().unwrap()).collect();
        assert!((positions[0] - 0.0).abs() < 1e-6);
        assert!((positions[1] - 1.0 / 3.0).abs() < 1e-6);
        assert!((positions[2] - 2.0 / 3.0).abs() < 1e-6);
        assert!((positions[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn linear_gradient_single_stop_is_unsupported() {
        // Fewer than two stops is not a valid gradient.
        assert!(matches!(
            compile("background-image: linear-gradient(red)").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }

    #[test]
    fn radial_gradient_bare_two_stops_defaults() {
        let g = radial_of("background-image: radial-gradient(red, blue)");
        assert_eq!(g.shape(), RadialShape::Ellipse);
        assert_eq!(g.size(), RadialSize::FarthestCorner);
        assert_eq!(g.center_x(), Length::Percent(50.0));
        assert_eq!(g.center_y(), Length::Percent(50.0));
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].color(), color(1.0, 0.0, 0.0));
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].color(), color(0.0, 0.0, 1.0));
        assert_eq!(g.stops()[1].position(), Some(1.0));
    }

    #[test]
    fn radial_gradient_circle_at_position_with_explicit_stops() {
        let g =
            radial_of("background-image: radial-gradient(circle at 30% 40%, red 0%, blue 100%)");
        assert_eq!(g.shape(), RadialShape::Circle);
        assert_eq!(g.center_x(), Length::Percent(30.0));
        assert_eq!(g.center_y(), Length::Percent(40.0));
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].position(), Some(1.0));
    }

    #[test]
    fn radial_gradient_extent_and_explicit_radius() {
        let g = radial_of("background-image: radial-gradient(closest-side, red, blue)");
        assert_eq!(g.size(), RadialSize::ClosestSide);
        let g2 = radial_of("background-image: radial-gradient(40px at center, red, blue)");
        assert_eq!(g2.size(), RadialSize::Length(Length::px(40.0)));
        assert_eq!(g2.center_x(), Length::Percent(50.0));
    }

    /// Checks explicit ellipse axis radii and rejects contradictory or excessive sizes.
    #[test]
    fn radial_gradient_preserves_both_explicit_axes() {
        let gradient =
            radial_of("background-image: radial-gradient(20px 30% ellipse at center, red, blue)");
        assert_eq!(gradient.shape(), RadialShape::Ellipse);
        assert_eq!(
            gradient.size(),
            RadialSize::Ellipse(Length::px(20.0), Length::Percent(30.0)),
        );
        for prelude in ["circle 20px 10px", "closest-side 20px", "20px 10px 5px"] {
            assert!(matches!(
                compile(&format!(
                    "background-image: radial-gradient({prelude}, red, blue)"
                ))
                .0,
                CompiledDeclaration::Unsupported(_)
            ));
        }
    }

    #[test]
    fn radial_gradient_malformed_is_unsupported() {
        // A single stop and an unrecognised prelude token both drop.
        assert!(matches!(
            compile("background-image: radial-gradient(red)").0,
            CompiledDeclaration::Unsupported(_)
        ));
        assert!(matches!(
            compile("background-image: radial-gradient(bogus at center, red, blue)").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }

    #[test]
    fn conic_gradient_from_angle_at_center() {
        let g = conic_of("background-image: conic-gradient(from 90deg at center, red, blue)");
        assert_eq!(g.from_angle_deg(), 90.0);
        assert_eq!(g.center_x(), Length::Percent(50.0));
        assert_eq!(g.center_y(), Length::Percent(50.0));
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].color(), color(1.0, 0.0, 0.0));
        assert_eq!(g.stops()[1].color(), color(0.0, 0.0, 1.0));
    }

    #[test]
    fn conic_gradient_bare_two_stops_defaults() {
        let g = conic_of("background-image: conic-gradient(red, blue)");
        assert_eq!(g.from_angle_deg(), 0.0);
        assert_eq!(g.center_x(), Length::Percent(50.0));
        assert_eq!(g.stops().len(), 2);
    }

    #[test]
    fn conic_gradient_malformed_is_unsupported() {
        assert!(matches!(
            compile("background-image: conic-gradient(red)").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }
}
