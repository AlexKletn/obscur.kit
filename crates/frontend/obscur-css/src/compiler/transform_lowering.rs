#![doc = include_str!("transform_lowering.doc/_module.md")]

use super::calc_lowering::lower_math_function;
use super::property_compiler::narrow_f32;
use super::value_lowering::{angle_to_degrees, dimension_length};
use crate::parser::ParsedValue;
use obscur_geometry::AffineTransform;
use obscur_style::property::{
    Length, LengthExpr, PropertyValue, Transform, TransformFunction, TransformOrigin,
};

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
    let mut functions: Vec<TransformFunction> = Vec::with_capacity(values.len());
    for value in values {
        let ParsedValue::Function { name, args } = value else {
            return None;
        };
        functions.push(lower_transform_function(name, args)?);
    }
    Some(flatten_or_defer(functions))
}

#[doc = include_str!("transform_lowering.doc/flatten_or_defer.md")]
fn flatten_or_defer(functions: Vec<TransformFunction>) -> PropertyValue {
    let mut acc = AffineTransform::IDENTITY;
    for function in &functions {
        let TransformFunction::Matrix(coeffs) = function else {
            return PropertyValue::TransformFunctions(functions.into_boxed_slice());
        };
        acc = acc.multiply(AffineTransform { coeffs: *coeffs });
    }
    PropertyValue::Transform(Transform::new(acc.coeffs))
}

#[doc = include_str!("transform_lowering.doc/lower_transform_function.md")]
fn lower_transform_function(name: &str, args: &[ParsedValue]) -> Option<TransformFunction> {
    let argument_groups: Vec<&[ParsedValue]> =
        args.split(|v| matches!(v, ParsedValue::Comma)).collect();
    if name.eq_ignore_ascii_case("translate") {
        return match argument_groups.as_slice() {
            [[x]] => Some(translate(translate_offset(x)?, zero())),
            [[x], [y]] => Some(translate(translate_offset(x)?, translate_offset(y)?)),
            _ => None,
        };
    } else if name.eq_ignore_ascii_case("translatex") {
        return Some(translate(
            translate_offset(single_arg(&argument_groups)?)?,
            zero(),
        ));
    } else if name.eq_ignore_ascii_case("translatey") {
        return Some(translate(
            zero(),
            translate_offset(single_arg(&argument_groups)?)?,
        ));
    }
    let affine = lower_affine_function(name, &argument_groups)?;
    Some(TransformFunction::Matrix(affine.coeffs))
}

#[doc = include_str!("transform_lowering.doc/lower_affine_function.md")]
// The `matrix(a, b, c, d, e, f)` arm binds the six canonical CSS coefficients.
#[allow(clippy::many_single_char_names)]
fn lower_affine_function(
    name: &str,
    argument_groups: &[&[ParsedValue]],
) -> Option<AffineTransform> {
    if name.eq_ignore_ascii_case("scale") {
        match argument_groups {
            [[s]] => {
                let s = arg_number(s)?;
                Some(AffineTransform::scale(s, s))
            }
            [[sx], [sy]] => Some(AffineTransform::scale(arg_number(sx)?, arg_number(sy)?)),
            _ => None,
        }
    } else if name.eq_ignore_ascii_case("scalex") {
        Some(AffineTransform::scale(
            arg_number(single_arg(argument_groups)?)?,
            1.0,
        ))
    } else if name.eq_ignore_ascii_case("scaley") {
        Some(AffineTransform::scale(
            1.0,
            arg_number(single_arg(argument_groups)?)?,
        ))
    } else if name.eq_ignore_ascii_case("rotate") {
        Some(AffineTransform::rotate(arg_angle_rad(single_arg(
            argument_groups,
        )?)?))
    } else if name.eq_ignore_ascii_case("skew") {
        match argument_groups {
            [[ax]] => Some(AffineTransform::skew(arg_angle_rad(ax)?, 0.0)),
            [[ax], [ay]] => Some(AffineTransform::skew(
                arg_angle_rad(ax)?,
                arg_angle_rad(ay)?,
            )),
            _ => None,
        }
    } else if name.eq_ignore_ascii_case("skewx") {
        Some(AffineTransform::skew(
            arg_angle_rad(single_arg(argument_groups)?)?,
            0.0,
        ))
    } else if name.eq_ignore_ascii_case("skewy") {
        Some(AffineTransform::skew(
            0.0,
            arg_angle_rad(single_arg(argument_groups)?)?,
        ))
    } else if name.eq_ignore_ascii_case("matrix") {
        match argument_groups {
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

#[doc = include_str!("transform_lowering.doc/zero.md")]
fn zero() -> LengthExpr {
    LengthExpr::leaf(Length::px(0.0))
}

#[doc = include_str!("transform_lowering.doc/translate.md")]
fn translate(x: LengthExpr, y: LengthExpr) -> TransformFunction {
    // A pair of plain pixels is already a matrix; collapsing it here keeps the
    // ordinary `translate(10px, 20px)` byte-identical to what it lowered to
    // before relative distances were carried at all.
    if let (LengthExpr::Leaf(Length::Px(x_px)), LengthExpr::Leaf(Length::Px(y_px))) = (&x, &y) {
        return TransformFunction::Matrix(AffineTransform::translate(x_px.0, y_px.0).coeffs);
    }
    TransformFunction::Translate(x, y)
}

#[doc = include_str!("transform_lowering.doc/translate_offset.md")]
// The percentage arm answers `None` like the wildcard does, and is spelled out
// anyway: it is the one rejection here that is a POLICY (see its comment), not
// a token this grammar never had a meaning for.
#[allow(clippy::match_same_arms)]
fn translate_offset(value: &ParsedValue) -> Option<LengthExpr> {
    match value {
        ParsedValue::Number(n) if *n == 0.0 => Some(zero()),
        // A percentage of the element's OWN border box, which is a used value:
        // no length context carries it, so it is refused here (the declaration
        // becomes an `UnsupportedCssFeature`) rather than silently taken as `0`.
        ParsedValue::Percentage(_) => None,
        ParsedValue::Dimension { value, unit } => match dimension_length(*value, unit)? {
            Length::Percent(_) => None,
            length => Some(LengthExpr::leaf(length)),
        },
        ParsedValue::Function { name, args } => {
            let expr = lower_math_function(name, args)?;
            (!expr.uses_percent()).then_some(expr)
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
    use super::super::property_compiler::tests::{compile, value_of};

    use crate::compiler::CompiledDeclaration;
    use obscur_style::property::{Length, LengthExpr, PropertyValue, TransformFunction};

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

    /// A percentage translate resolves against the element's OWN border box —
    /// a used value nothing here can reach. It used to be silently taken as `0`,
    /// which made the standard centring idiom do nothing with no error to grep
    /// for. It now reports itself as an unsupported feature.
    #[test]
    fn transform_percent_translate_is_reported_not_zeroed() {
        for text in [
            "transform: translate(-50%, -50%)",
            "transform: translateX(100%)",
            "transform: translateY(50%)",
            // Hidden inside a math function, it is the same unknown.
            "transform: translateX(calc(100% - 10px))",
        ] {
            assert!(
                matches!(compile(text).0, CompiledDeclaration::Unsupported(_)),
                "{text} must be reported, never silently zeroed"
            );
        }
    }

    /// The other half of the same defect: a relative distance used to take the
    /// WHOLE declaration down (no transform at all, no matrix, no diagnostic
    /// naming the unit). It now survives lowering unresolved and is folded at
    /// computed-value time.
    #[test]
    fn transform_relative_translate_survives_as_an_unresolved_function_list() {
        let PropertyValue::TransformFunctions(list) = value_of("transform: translate(2em, 10vw)")
        else {
            panic!("a relative translate is carried, not flattened");
        };
        assert_eq!(
            &*list,
            &[TransformFunction::Translate(
                LengthExpr::leaf(Length::Em(2.0)),
                LengthExpr::leaf(Length::Vw(10.0)),
            )]
        );
        // …and it composes with the absolute functions around it, in order.
        let PropertyValue::TransformFunctions(chain) =
            value_of("transform: scale(2) translateY(1rem)")
        else {
            panic!("the whole list is carried once one function is unresolved");
        };
        assert_eq!(chain.len(), 2);
        assert_eq!(
            chain[0],
            TransformFunction::Matrix([2.0, 0.0, 0.0, 2.0, 0.0, 0.0])
        );
        assert_eq!(
            chain[1],
            TransformFunction::Translate(
                LengthExpr::leaf(Length::px(0.0)),
                LengthExpr::leaf(Length::Rem(1.0)),
            )
        );
    }

    #[test]
    fn transform_translate_calc_is_carried_as_the_expression() {
        let PropertyValue::TransformFunctions(list) =
            value_of("transform: translateX(calc(1rem + 4px))")
        else {
            panic!("a calc distance is carried, not dropped");
        };
        let TransformFunction::Translate(x, _) = &list[0] else {
            panic!("the x distance is the expression");
        };
        assert_eq!(
            x,
            &LengthExpr::Sum(
                Box::new(LengthExpr::leaf(Length::Rem(1.0))),
                Box::new(LengthExpr::leaf(Length::px(4.0))),
            )
        );
    }

    #[test]
    fn transform_absolute_units_still_flatten_at_lower_time() {
        // `pt`/`cm`/… are absolute, so they compose here and never pay for the
        // unresolved list: 12pt = 16px.
        matrix_near(
            transform_of("transform: translateX(12pt)"),
            [1.0, 0.0, 0.0, 1.0, 16.0, 0.0],
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
