#![doc = include_str!("transform_lowering.doc/_module.md")]

use super::property_compiler::narrow_f32;
use super::value_lowering::{angle_to_degrees, dimension_length};
use crate::parser::ParsedValue;
use obscur_geometry::AffineTransform;
use obscur_style::property::{Length, PropertyValue, Transform, TransformOrigin};

#[doc = include_str!("transform_lowering.doc/lower_transform.md")]
pub(super) fn lower_transform(values: &[ParsedValue]) -> Option<PropertyValue> {
    if let [ParsedValue::Keyword(keyword)] = values
        && keyword.eq_ignore_ascii_case("none")
    {
        return Some(PropertyValue::Transform(Transform::NONE));
    }
    if values.is_empty() {
        return None;
    }
    let mut acc = AffineTransform::IDENTITY;
    for value in values {
        let ParsedValue::Function { name, args } = value else {
            return None;
        };
        acc = acc.multiply(lower_transform_function(name, args)?);
    }
    Some(PropertyValue::Transform(Transform::new(acc.coeffs)))
}

#[doc = include_str!("transform_lowering.doc/lower_transform_function.md")]
// The `matrix(a, b, c, d, e, f)` arm binds the six canonical CSS coefficients.
#[allow(clippy::many_single_char_names)]
fn lower_transform_function(name: &str, args: &[ParsedValue]) -> Option<AffineTransform> {
    let argument_groups: Vec<&[ParsedValue]> =
        args.split(|v| matches!(v, ParsedValue::Comma)).collect();
    if name.eq_ignore_ascii_case("translate") {
        match argument_groups.as_slice() {
            [[x]] => Some(AffineTransform::translate(translate_px(x)?, 0.0)),
            [[x], [y]] => Some(AffineTransform::translate(
                translate_px(x)?,
                translate_px(y)?,
            )),
            _ => None,
        }
    } else if name.eq_ignore_ascii_case("translatex") {
        Some(AffineTransform::translate(
            translate_px(single_arg(&argument_groups)?)?,
            0.0,
        ))
    } else if name.eq_ignore_ascii_case("translatey") {
        Some(AffineTransform::translate(
            0.0,
            translate_px(single_arg(&argument_groups)?)?,
        ))
    } else if name.eq_ignore_ascii_case("scale") {
        match argument_groups.as_slice() {
            [[s]] => {
                let s = arg_number(s)?;
                Some(AffineTransform::scale(s, s))
            }
            [[sx], [sy]] => Some(AffineTransform::scale(arg_number(sx)?, arg_number(sy)?)),
            _ => None,
        }
    } else if name.eq_ignore_ascii_case("scalex") {
        Some(AffineTransform::scale(
            arg_number(single_arg(&argument_groups)?)?,
            1.0,
        ))
    } else if name.eq_ignore_ascii_case("scaley") {
        Some(AffineTransform::scale(
            1.0,
            arg_number(single_arg(&argument_groups)?)?,
        ))
    } else if name.eq_ignore_ascii_case("rotate") {
        Some(AffineTransform::rotate(arg_angle_rad(single_arg(
            &argument_groups,
        )?)?))
    } else if name.eq_ignore_ascii_case("skew") {
        match argument_groups.as_slice() {
            [[ax]] => Some(AffineTransform::skew(arg_angle_rad(ax)?, 0.0)),
            [[ax], [ay]] => Some(AffineTransform::skew(
                arg_angle_rad(ax)?,
                arg_angle_rad(ay)?,
            )),
            _ => None,
        }
    } else if name.eq_ignore_ascii_case("skewx") {
        Some(AffineTransform::skew(
            arg_angle_rad(single_arg(&argument_groups)?)?,
            0.0,
        ))
    } else if name.eq_ignore_ascii_case("skewy") {
        Some(AffineTransform::skew(
            0.0,
            arg_angle_rad(single_arg(&argument_groups)?)?,
        ))
    } else if name.eq_ignore_ascii_case("matrix") {
        match argument_groups.as_slice() {
            [[a], [b], [c], [d], [e], [f]] => Some(AffineTransform::from_matrix(
                arg_number(a)?,
                arg_number(b)?,
                arg_number(c)?,
                arg_number(d)?,
                arg_number(e)?,
                arg_number(f)?,
            )),
            _ => None,
        }
    } else {
        None
    }
}

#[doc = include_str!("transform_lowering.doc/single_arg.md")]
fn single_arg<'a>(groups: &[&'a [ParsedValue]]) -> Option<&'a ParsedValue> {
    match groups {
        [[v]] => Some(v),
        _ => None,
    }
}

#[doc = include_str!("transform_lowering.doc/translate_px.md")]
fn translate_px(value: &ParsedValue) -> Option<f32> {
    match value {
        ParsedValue::Number(n) if *n == 0.0 => Some(0.0),
        ParsedValue::Percentage(_) => Some(0.0),
        ParsedValue::Dimension { value, unit } if unit.eq_ignore_ascii_case("px") => {
            Some(narrow_f32(*value))
        }
        _ => None,
    }
}

#[doc = include_str!("transform_lowering.doc/arg_number.md")]
fn arg_number(value: &ParsedValue) -> Option<f32> {
    match value {
        ParsedValue::Number(n) => Some(narrow_f32(*n)),
        _ => None,
    }
}

#[doc = include_str!("transform_lowering.doc/arg_angle_rad.md")]
fn arg_angle_rad(value: &ParsedValue) -> Option<f32> {
    match value {
        ParsedValue::Number(n) if *n == 0.0 => Some(0.0),
        ParsedValue::Dimension { value, unit } => {
            angle_to_degrees(*value, unit).map(f32::to_radians)
        }
        _ => None,
    }
}

#[doc = include_str!("transform_lowering.doc/lower_transform_origin.md")]
pub(super) fn lower_transform_origin(values: &[ParsedValue]) -> Option<PropertyValue> {
    let center = Length::Percent(50.0);
    let origin = match values {
        [a] => {
            let (len, axis) = origin_component(a)?;
            match axis {
                OriginAxis::Y => TransformOrigin::new(center, len),
                _ => TransformOrigin::new(len, center),
            }
        }
        [a, b] => {
            let ac = origin_component(a)?;
            let bc = origin_component(b)?;
            // Accept `top left` order: a vertical first / horizontal second swaps.
            let (x, y) = if ac.1 == OriginAxis::Y || bc.1 == OriginAxis::X {
                (bc.0, ac.0)
            } else {
                (ac.0, bc.0)
            };
            TransformOrigin::new(x, y)
        }
        _ => return None,
    };
    Some(PropertyValue::TransformOrigin(origin))
}

#[doc = include_str!("transform_lowering.doc/OriginAxis.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OriginAxis {
    #[doc = include_str!("transform_lowering.doc/X.md")]
    X,
    #[doc = include_str!("transform_lowering.doc/Y.md")]
    Y,
    #[doc = include_str!("transform_lowering.doc/Either.md")]
    Either,
}

#[doc = include_str!("transform_lowering.doc/origin_component.md")]
pub(super) fn origin_component(value: &ParsedValue) -> Option<(Length, OriginAxis)> {
    match value {
        ParsedValue::Number(n) if *n == 0.0 => Some((Length::px(0.0), OriginAxis::Either)),
        ParsedValue::Percentage(p) => Some((Length::Percent(narrow_f32(*p)), OriginAxis::Either)),
        ParsedValue::Dimension { value, unit } => {
            dimension_length(*value, unit).map(|len| (len, OriginAxis::Either))
        }
        ParsedValue::Keyword(keyword) => match keyword.to_ascii_lowercase().as_str() {
            "left" => Some((Length::Percent(0.0), OriginAxis::X)),
            "right" => Some((Length::Percent(100.0), OriginAxis::X)),
            "top" => Some((Length::Percent(0.0), OriginAxis::Y)),
            "bottom" => Some((Length::Percent(100.0), OriginAxis::Y)),
            "center" => Some((Length::Percent(50.0), OriginAxis::Either)),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::property_compiler::tests::value_of;

    use obscur_style::property::{Length, PropertyValue};

    /// Extracts the lowered resolved affine `[a,b,c,d,e,f]` from a `transform`.
    fn transform_of(text: &str) -> [f32; 6] {
        match value_of(text) {
            PropertyValue::Transform(t) => t.matrix(),
            other => panic!("expected a transform, got {other:?}"),
        }
    }

    fn matrix_near(a: [f32; 6], b: [f32; 6]) {
        for (x, y) in a.iter().zip(b) {
            assert!((x - y).abs() < 1e-4, "matrix mismatch: {a:?} vs {b:?}");
        }
    }

    #[test]
    fn transform_translate_sets_e_and_f() {
        matrix_near(
            transform_of("transform: translate(10px, 20px)"),
            [1.0, 0.0, 0.0, 1.0, 10.0, 20.0],
        );
        // translateX / translateY set one axis.
        matrix_near(
            transform_of("transform: translateX(7px)"),
            [1.0, 0.0, 0.0, 1.0, 7.0, 0.0],
        );
        matrix_near(
            transform_of("transform: translateY(9px)"),
            [1.0, 0.0, 0.0, 1.0, 0.0, 9.0],
        );
    }

    #[test]
    fn transform_scale_sets_a_and_d() {
        matrix_near(
            transform_of("transform: scale(2)"),
            [2.0, 0.0, 0.0, 2.0, 0.0, 0.0],
        );
        matrix_near(
            transform_of("transform: scale(2, 3)"),
            [2.0, 0.0, 0.0, 3.0, 0.0, 0.0],
        );
    }

    #[test]
    fn transform_rotate_90deg_is_rotation_matrix() {
        // rotate(90deg): cos=0, sin=1 → [0, 1, -1, 0, 0, 0] (y-down space).
        matrix_near(
            transform_of("transform: rotate(90deg)"),
            [0.0, 1.0, -1.0, 0.0, 0.0, 0.0],
        );
    }

    #[test]
    fn transform_matrix_is_passed_through() {
        matrix_near(
            transform_of("transform: matrix(1, 0, 0, 1, 5, 6)"),
            [1.0, 0.0, 0.0, 1.0, 5.0, 6.0],
        );
    }

    #[test]
    fn transform_composes_left_to_right() {
        // `translate(10px) scale(2)`: composed M = translate · scale, so a point
        // is scaled first then translated. e/f are the translate (10, 0), a/d are
        // the scale (2, 2). A different order (scale then translate) would scale
        // the translation into e=20.
        matrix_near(
            transform_of("transform: translate(10px) scale(2)"),
            [2.0, 0.0, 0.0, 2.0, 10.0, 0.0],
        );
        matrix_near(
            transform_of("transform: scale(2) translate(10px)"),
            [2.0, 0.0, 0.0, 2.0, 20.0, 0.0],
        );
    }

    #[test]
    fn transform_none_is_identity() {
        matrix_near(
            transform_of("transform: none"),
            [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        );
    }

    #[test]
    fn transform_percent_translate_is_zero_approximation() {
        // A percentage translate is treated as 0 (box size unknown at lower time).
        matrix_near(
            transform_of("transform: translate(50%, 50%)"),
            [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        );
    }

    #[test]
    fn transform_origin_center_is_fifty_percent() {
        let origin = match value_of("transform-origin: center") {
            PropertyValue::TransformOrigin(o) => o,
            other => panic!("expected a transform-origin, got {other:?}"),
        };
        assert_eq!(origin.x(), Length::Percent(50.0));
        assert_eq!(origin.y(), Length::Percent(50.0));
    }

    #[test]
    fn transform_origin_two_values_and_keyword_order() {
        let a = match value_of("transform-origin: 10px 20px") {
            PropertyValue::TransformOrigin(o) => o,
            other => panic!("got {other:?}"),
        };
        assert_eq!(a.x(), Length::px(10.0));
        assert_eq!(a.y(), Length::px(20.0));
        // `top left` (vertical first) is swapped to x=left, y=top.
        let b = match value_of("transform-origin: top left") {
            PropertyValue::TransformOrigin(o) => o,
            other => panic!("got {other:?}"),
        };
        assert_eq!(b.x(), Length::Percent(0.0));
        assert_eq!(b.y(), Length::Percent(0.0));
    }
}
