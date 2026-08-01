#![doc = include_str!("filter_lowering.doc/_module.md")]

use super::color_lowering::parse_stop_color;
use super::property_compiler::narrow_f32;
use super::shadow_lowering::shadow_length;
use crate::parser::ParsedValue;
use obscur_style::property::{BoxShadow, FilterOp, Length, PropertyValue, StyleColor};

#[doc = include_str!("filter_lowering.doc/lower_filter.md")]
pub(super) fn lower_filter(values: &[ParsedValue]) -> Option<PropertyValue> {
    // `none` → an empty op list.
    if let [ParsedValue::Keyword(keyword)] = values
        && keyword.eq_ignore_ascii_case("none")
    {
        return Some(PropertyValue::Filter(Box::new([])));
    }
    let mut ops: Vec<FilterOp> = Vec::new();
    for value in values {
        let ParsedValue::Function { name, args } = value else {
            // A bare token in the filter list is not a filter function; the value
            // is malformed.
            return None;
        };
        if name.eq_ignore_ascii_case("drop-shadow") {
            ops.push(FilterOp::DropShadow(lower_one_drop_shadow(args)?));
        } else if name.eq_ignore_ascii_case("blur") {
            // A positive radius becomes a blur op; `blur(0)` / `blur()` is a no-op
            // (kept out of the list so it stays byte-identical to no filter).
            let radius = lower_one_blur(args)?;
            if radius > 0.0 {
                ops.push(FilterOp::Blur(radius));
            }
        } else if is_color_filter_name(name) {
            // A color-matrix function (`brightness`/`contrast`/`saturate`/
            // `grayscale`/`sepia`/`invert`/`hue-rotate`/`opacity`): each is an
            // affine color transform the coordinator folds into ONE 4×5 matrix.
            ops.push(lower_one_color_filter(name, args)?);
        }
        // Every other (unrecognised) filter function is out of scope and silently
        // dropped — not faked.
    }
    Some(PropertyValue::Filter(ops.into_boxed_slice()))
}

#[doc = include_str!("filter_lowering.doc/is_color_filter_name.md")]
fn is_color_filter_name(name: &str) -> bool {
    [
        "brightness",
        "contrast",
        "saturate",
        "grayscale",
        "sepia",
        "invert",
        "opacity",
        "hue-rotate",
    ]
    .iter()
    .any(|known| name.eq_ignore_ascii_case(known))
}

#[doc = include_str!("filter_lowering.doc/lower_one_color_filter.md")]
fn lower_one_color_filter(name: &str, args: &[ParsedValue]) -> Option<FilterOp> {
    // Clamp helper for the `0..1` amount functions (grayscale/sepia/invert/opacity).
    let unit = |amount: f32| amount.clamp(0.0, 1.0);
    // Clamp helper for the unbounded-above, `>= 0` functions
    // (brightness/contrast/saturate): negatives are clamped to `0`.
    let non_negative = |amount: f32| amount.max(0.0);

    Some(if name.eq_ignore_ascii_case("brightness") {
        FilterOp::Brightness(non_negative(filter_amount(args, 1.0)?))
    } else if name.eq_ignore_ascii_case("contrast") {
        FilterOp::Contrast(non_negative(filter_amount(args, 1.0)?))
    } else if name.eq_ignore_ascii_case("saturate") {
        FilterOp::Saturate(non_negative(filter_amount(args, 1.0)?))
    } else if name.eq_ignore_ascii_case("grayscale") {
        FilterOp::Grayscale(unit(filter_amount(args, 1.0)?))
    } else if name.eq_ignore_ascii_case("sepia") {
        FilterOp::Sepia(unit(filter_amount(args, 1.0)?))
    } else if name.eq_ignore_ascii_case("invert") {
        FilterOp::Invert(unit(filter_amount(args, 1.0)?))
    } else if name.eq_ignore_ascii_case("hue-rotate") {
        FilterOp::HueRotate(filter_angle_radians(args)?)
    } else {
        // `opacity` (the remaining color function per `is_color_filter_name`).
        FilterOp::Opacity(unit(filter_amount(args, 1.0)?))
    })
}

#[doc = include_str!("filter_lowering.doc/filter_amount.md")]
fn filter_amount(args: &[ParsedValue], default: f32) -> Option<f32> {
    if args.is_empty() {
        return Some(default);
    }
    let [value] = args else {
        return None;
    };
    match value {
        ParsedValue::Number(number) => Some(narrow_f32(*number)),
        ParsedValue::Percentage(percentage) => Some(narrow_f32(*percentage / 100.0)),
        _ => None,
    }
}

#[doc = include_str!("filter_lowering.doc/filter_angle_radians.md")]
fn filter_angle_radians(args: &[ParsedValue]) -> Option<f32> {
    use core::f32::consts::PI;
    if args.is_empty() {
        return Some(0.0);
    }
    let [value] = args else {
        return None;
    };
    match value {
        // A bare unitless angle is only valid as `0`.
        ParsedValue::Number(number) if *number == 0.0 => Some(0.0),
        ParsedValue::Dimension { value, unit } => {
            let degrees = narrow_f32(*value);
            if unit.eq_ignore_ascii_case("deg") {
                Some(degrees.to_radians())
            } else if unit.eq_ignore_ascii_case("rad") {
                Some(degrees)
            } else if unit.eq_ignore_ascii_case("grad") {
                Some(degrees * PI / 200.0)
            } else if unit.eq_ignore_ascii_case("turn") {
                Some(degrees * 2.0 * PI)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[doc = include_str!("filter_lowering.doc/lower_backdrop_filter.md")]
pub(super) fn lower_backdrop_filter(values: &[ParsedValue]) -> Option<PropertyValue> {
    // `none` → an empty op list.
    if let [ParsedValue::Keyword(keyword)] = values
        && keyword.eq_ignore_ascii_case("none")
    {
        return Some(PropertyValue::Filter(Box::new([])));
    }
    let mut ops: Vec<FilterOp> = Vec::new();
    for value in values {
        let ParsedValue::Function { name, args } = value else {
            // A bare token in the list is not a filter function; it is malformed.
            return None;
        };
        if name.eq_ignore_ascii_case("blur") {
            // A positive radius becomes a blur op; `blur(0)` / `blur()` is a no-op
            // (kept out of the list so it stays byte-identical to no effect).
            let radius = lower_one_blur(args)?;
            if radius > 0.0 {
                ops.push(FilterOp::Blur(radius));
            }
        }
        // Every other function (color-matrix, `drop-shadow()`, `brightness()`, …)
        // is out of scope for `backdrop-filter` and silently dropped — not faked.
    }
    Some(PropertyValue::Filter(ops.into_boxed_slice()))
}

#[doc = include_str!("filter_lowering.doc/lower_one_blur.md")]
fn lower_one_blur(args: &[ParsedValue]) -> Option<f32> {
    // `blur()` → 0 (the CSS initial): a no-op.
    if args.is_empty() {
        return Some(0.0);
    }
    let [value] = args else {
        return None;
    };
    let length = shadow_length(value)?;
    let px = length.as_px().map_or(0.0, |px| px.0);
    if px < 0.0 {
        return None;
    }
    Some(px)
}

#[doc = include_str!("filter_lowering.doc/lower_one_drop_shadow.md")]
fn lower_one_drop_shadow(args: &[ParsedValue]) -> Option<BoxShadow> {
    let mut lengths: Vec<Length> = Vec::new();
    let mut color: Option<StyleColor> = None;
    for value in args {
        if let Some(length) = shadow_length(value) {
            if lengths.len() == 3 {
                return None;
            }
            lengths.push(length);
            continue;
        }
        if let Some(parsed) = parse_stop_color(value) {
            if color.is_some() {
                return None;
            }
            color = Some(parsed);
            continue;
        }
        // A comma (or any other token) is invalid inside `drop-shadow()`.
        return None;
    }
    // Two required offsets, then an optional blur; spread is fixed at `0` and the
    // shadow is never inset (a drop-shadow is a drop-only box-shadow).
    let (offset_x, offset_y, blur) = match lengths.as_slice() {
        [x, y] => (*x, *y, Length::px(0.0)),
        [x, y, b] => (*x, *y, *b),
        _ => return None,
    };
    Some(BoxShadow::new(
        offset_x,
        offset_y,
        blur,
        Length::px(0.0),
        color,
        false,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::color_lowering::tests::color;
    use super::super::property_compiler::tests::{compile, value_of};

    use crate::compiler::CompiledDeclaration;

    use obscur_style::property::{BoxShadow, FilterOp, Length, PropertyValue, StyleColor};

    /// Extracts the lowered ordered `filter` op list from a declaration.
    fn filter_ops_of(text: &str) -> Box<[FilterOp]> {
        match value_of(text) {
            PropertyValue::Filter(list) => list,
            other => panic!("expected a filter op list, got {other:?}"),
        }
    }

    /// Extracts only the drop-shadow ops (in order) from a lowered `filter`.
    fn drop_shadows_of(text: &str) -> Vec<BoxShadow> {
        filter_ops_of(text)
            .iter()
            .filter_map(FilterOp::as_drop_shadow)
            .cloned()
            .collect()
    }

    #[test]
    fn filter_drop_shadow_offset_blur_color() {
        // `drop-shadow(4px 4px 8px rgba(0,0,0,0.5))` → one drop shadow: offset
        // (4,4), blur 8, spread 0, the color, not inset.
        let list = drop_shadows_of("filter: drop-shadow(4px 4px 8px rgba(0,0,0,0.5))");
        assert_eq!(list.len(), 1);
        let s = &list[0];
        assert_eq!(s.offset_x(), Length::px(4.0));
        assert_eq!(s.offset_y(), Length::px(4.0));
        assert_eq!(s.blur(), Length::px(8.0));
        assert_eq!(s.spread(), Length::px(0.0));
        assert_eq!(s.color(), Some(StyleColor::rgba(0.0, 0.0, 0.0, 0.5)));
        assert!(!s.is_inset());
    }

    #[test]
    fn filter_multiple_drop_shadows_preserve_order() {
        // `drop-shadow(2px 2px) drop-shadow(-2px -2px 4px red)` → two shadows.
        let list = drop_shadows_of("filter: drop-shadow(2px 2px) drop-shadow(-2px -2px 4px red)");
        assert_eq!(list.len(), 2);
        // First: no blur, currentColor default.
        assert_eq!(list[0].offset_x(), Length::px(2.0));
        assert_eq!(list[0].offset_y(), Length::px(2.0));
        assert_eq!(list[0].blur(), Length::px(0.0));
        assert_eq!(list[0].color(), None);
        assert!(!list[0].is_inset());
        // Second: negative offsets, blur, red.
        assert_eq!(list[1].offset_x(), Length::px(-2.0));
        assert_eq!(list[1].offset_y(), Length::px(-2.0));
        assert_eq!(list[1].blur(), Length::px(4.0));
        assert_eq!(list[1].color(), Some(color(1.0, 0.0, 0.0)));
    }

    #[test]
    fn filter_blur_lowers_to_a_blur_op() {
        // `blur(8px)` now lowers to a single `Blur(8)` op (content blur), not a
        // silently-dropped no-op.
        let ops = filter_ops_of("filter: blur(8px)");
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0], FilterOp::Blur(8.0));
    }

    #[test]
    fn filter_blur_then_drop_shadow_keeps_both_in_order() {
        // `blur(4px) drop-shadow(...)` lowers to two ops in author order: the blur
        // first, then the drop-shadow.
        let ops = filter_ops_of("filter: blur(4px) drop-shadow(1px 1px 2px black)");
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0], FilterOp::Blur(4.0));
        let shadow = ops[1].as_drop_shadow().expect("second op is a drop-shadow");
        assert_eq!(shadow.offset_x(), Length::px(1.0));
        assert_eq!(shadow.color(), Some(StyleColor::BLACK));
    }

    #[test]
    fn filter_zero_blur_is_a_no_op() {
        // `blur(0)` / `blur()` carries no op, keeping the byte-identical no-blur path.
        assert!(filter_ops_of("filter: blur(0)").is_empty());
        assert!(filter_ops_of("filter: blur()").is_empty());
    }

    #[test]
    fn filter_unknown_function_is_dropped() {
        // A genuinely unrecognised function is silently dropped: it has no
        // GPU-expressible path, so the lowered op list is empty (not an error).
        assert!(filter_ops_of("filter: contrastxyz(0.5)").is_empty());
    }

    #[test]
    fn filter_color_functions_lower_to_color_ops() {
        // Numbers and percentages both parse (100% == 1); a missing amount is the
        // per-function default (1 for these, 0deg for hue-rotate).
        assert_eq!(
            filter_ops_of("filter: brightness(1.5)")[0],
            FilterOp::Brightness(1.5)
        );
        assert_eq!(
            filter_ops_of("filter: brightness(150%)")[0],
            FilterOp::Brightness(1.5)
        );
        assert_eq!(
            filter_ops_of("filter: brightness()")[0],
            FilterOp::Brightness(1.0)
        );
        assert_eq!(
            filter_ops_of("filter: contrast(0.5)")[0],
            FilterOp::Contrast(0.5)
        );
        assert_eq!(
            filter_ops_of("filter: saturate(2)")[0],
            FilterOp::Saturate(2.0)
        );
        assert_eq!(
            filter_ops_of("filter: grayscale(50%)")[0],
            FilterOp::Grayscale(0.5)
        );
        assert_eq!(
            filter_ops_of("filter: grayscale()")[0],
            FilterOp::Grayscale(1.0)
        );
        assert_eq!(filter_ops_of("filter: sepia(1)")[0], FilterOp::Sepia(1.0));
        assert_eq!(
            filter_ops_of("filter: invert(100%)")[0],
            FilterOp::Invert(1.0)
        );
        assert_eq!(
            filter_ops_of("filter: opacity(0.25)")[0],
            FilterOp::Opacity(0.25)
        );
        // Amounts clamp to their spec ranges.
        assert_eq!(
            filter_ops_of("filter: grayscale(200%)")[0],
            FilterOp::Grayscale(1.0)
        );
        assert_eq!(
            filter_ops_of("filter: brightness(-1)")[0],
            FilterOp::Brightness(0.0)
        );
    }

    #[test]
    fn filter_hue_rotate_lowers_to_radians() {
        // `hue-rotate` stores radians; deg/grad/rad/turn and a bare 0 all parse.
        let deg = filter_ops_of("filter: hue-rotate(90deg)");
        let FilterOp::HueRotate(r) = deg[0] else {
            panic!("hue-rotate")
        };
        assert!((r - core::f32::consts::FRAC_PI_2).abs() < 1e-5);
        let turn = filter_ops_of("filter: hue-rotate(0.5turn)");
        let FilterOp::HueRotate(r) = turn[0] else {
            panic!("hue-rotate")
        };
        assert!((r - core::f32::consts::PI).abs() < 1e-5);
        assert_eq!(
            filter_ops_of("filter: hue-rotate()")[0],
            FilterOp::HueRotate(0.0)
        );
        assert_eq!(
            filter_ops_of("filter: hue-rotate(0)")[0],
            FilterOp::HueRotate(0.0)
        );
    }

    #[test]
    fn filter_color_op_chain_preserves_order() {
        // A mixed chain keeps every op in author order (spatial + color coexist).
        let ops = filter_ops_of("filter: grayscale(1) brightness(1.2) blur(3px)");
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0], FilterOp::Grayscale(1.0));
        assert_eq!(ops[1], FilterOp::Brightness(1.2));
        assert_eq!(ops[2], FilterOp::Blur(3.0));
    }

    #[test]
    fn filter_none_is_an_empty_list() {
        assert!(filter_ops_of("filter: none").is_empty());
    }

    #[test]
    fn backdrop_filter_blur_lowers_to_a_blur_op() {
        // `backdrop-filter: blur(12px)` lowers to a single `Blur(12)` op (the
        // frosted-glass radius the coordinator emits as a BackdropFilter command).
        let ops = filter_ops_of("backdrop-filter: blur(12px)");
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0], FilterOp::Blur(12.0));
    }

    #[test]
    fn backdrop_filter_zero_blur_and_none_are_empty() {
        // `blur(0)` / `blur()` / `none` all carry no op (byte-identical no-effect).
        assert!(filter_ops_of("backdrop-filter: blur(0)").is_empty());
        assert!(filter_ops_of("backdrop-filter: blur()").is_empty());
        assert!(filter_ops_of("backdrop-filter: none").is_empty());
    }

    #[test]
    fn backdrop_filter_other_functions_are_dropped() {
        // Only `blur()` is honored; a `drop-shadow()` / `brightness()` is out of
        // scope for the backdrop pass and silently dropped (empty list, not error).
        assert!(filter_ops_of("backdrop-filter: drop-shadow(2px 2px 2px black)").is_empty());
        assert!(filter_ops_of("backdrop-filter: brightness(0.5)").is_empty());
    }

    #[test]
    fn filter_malformed_drop_shadow_is_unsupported() {
        // A `drop-shadow()` with a single offset is malformed → unsupported, not a
        // silent empty filter.
        assert!(matches!(
            compile("filter: drop-shadow(4px)").0,
            CompiledDeclaration::Unsupported(_)
        ));
    }
}
