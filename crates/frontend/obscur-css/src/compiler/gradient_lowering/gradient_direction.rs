use obscur_style::property::{Corner, GradientStop, LinearGradient};

use super::super::value_lowering::angle_to_degrees;
use crate::parser::ParsedValue;

/// The resolved direction of a `linear-gradient`: either an explicit `<angle>`/
/// `to <side>` (fully defined by a CSS-convention angle) or a `to <corner>` (the
/// [`Corner`] plus its nominal diagonal angle, resolved aspect-ratio-aware at paint
/// time per spec §3.1).
#[derive(Clone, Copy)]
pub(super) enum GradientDirection {
    /// An angle in CSS-convention degrees (`0` = to top, increasing clockwise).
    Angle(f32),
    /// A `to <corner>` direction, carrying the corner and its nominal angle.
    Corner(Corner, f32),
}

impl GradientDirection {
    /// The nominal angle in degrees (the diagonal angle for a corner).
    pub(super) fn nominal_angle(self) -> f32 {
        match self {
            Self::Angle(a) | Self::Corner(_, a) => a,
        }
    }

    /// Builds a non-repeating [`LinearGradient`] for this direction and `stops`.
    pub(super) fn into_linear(self, stops: Vec<GradientStop>) -> LinearGradient {
        match self {
            Self::Angle(a) => LinearGradient::new(a, stops),
            Self::Corner(c, a) => LinearGradient::new_corner(c, a, stops),
        }
    }
}

/// Parses the optional leading direction argument of a `linear-gradient`: a single
/// `<angle>` dimension, or a `to <side>` / `to <corner>` keyword form. Returns
/// [`None`] when the part is not a direction (so it is treated as the first color
/// stop instead).
pub(super) fn parse_gradient_direction(part: &[ParsedValue]) -> Option<GradientDirection> {
    match part {
        [ParsedValue::Dimension { value, unit }] => {
            angle_to_degrees(*value, unit).map(GradientDirection::Angle)
        }
        [ParsedValue::Keyword(to), rest @ ..] if to.eq_ignore_ascii_case("to") => {
            to_side_direction(rest)
        }
        _ => None,
    }
}

/// Resolves a `to <side>` / `to <corner>` keyword list (the tokens after `to`)
/// into a [`GradientDirection`]. A single side maps to its axis angle; a two-keyword
/// corner maps to a [`Corner`] (carrying its nominal diagonal angle, resolved
/// aspect-ratio-aware at paint time). Returns [`None`] for an empty or unrecognised
/// list.
fn to_side_direction(sides: &[ParsedValue]) -> Option<GradientDirection> {
    let mut names: Vec<String> = Vec::new();
    for side in sides {
        match side {
            ParsedValue::Keyword(keyword) => names.push(keyword.to_ascii_lowercase()),
            _ => return None,
        }
    }
    match names.as_slice() {
        [a] => single_side_angle(a).map(GradientDirection::Angle),
        [a, b] => corner_direction(a, b),
        _ => None,
    }
}

/// The CSS-convention angle of a single `to <side>` keyword.
fn single_side_angle(side: &str) -> Option<f32> {
    match side {
        "top" => Some(0.0),
        "right" => Some(90.0),
        "bottom" => Some(180.0),
        "left" => Some(270.0),
        _ => None,
    }
}

/// Resolves a `to <corner>` keyword pair to its [`Corner`] and nominal diagonal
/// angle (the aspect-ratio-aware angle is computed against the box at paint time;
/// the nominal angle is a square-box fallback). The two keywords must name one
/// vertical (`top` / `bottom`) and one horizontal (`left` / `right`) edge, in
/// either order.
fn corner_direction(a: &str, b: &str) -> Option<GradientDirection> {
    if a == b {
        return None;
    }
    let has = |name: &str| a == name || b == name;
    let (corner, nominal) = if has("top") && has("right") {
        (Corner::TopRight, 45.0)
    } else if has("bottom") && has("right") {
        (Corner::BottomRight, 135.0)
    } else if has("bottom") && has("left") {
        (Corner::BottomLeft, 225.0)
    } else if has("top") && has("left") {
        (Corner::TopLeft, 315.0)
    } else {
        return None;
    };
    Some(GradientDirection::Corner(corner, nominal))
}
