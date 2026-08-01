#![doc = include_str!("skia_lowering.doc/_module.md")]

use obscur_geometry::{AffineTransform, Size};
use obscur_scene::color::Color;
use obscur_scene::paint::BlendMode;
use obscur_scene::path::{FillRule, Path, PathSegment};

#[doc = include_str!("skia_lowering.doc/to_sk_blend_mode.md")]
pub(crate) fn to_sk_blend_mode(mode: BlendMode) -> tiny_skia::BlendMode {
    match mode {
        BlendMode::Normal => tiny_skia::BlendMode::SourceOver,
        BlendMode::Multiply => tiny_skia::BlendMode::Multiply,
        BlendMode::Screen => tiny_skia::BlendMode::Screen,
        BlendMode::Overlay => tiny_skia::BlendMode::Overlay,
        BlendMode::Darken => tiny_skia::BlendMode::Darken,
        BlendMode::Lighten => tiny_skia::BlendMode::Lighten,
        BlendMode::ColorDodge => tiny_skia::BlendMode::ColorDodge,
        BlendMode::ColorBurn => tiny_skia::BlendMode::ColorBurn,
        BlendMode::HardLight => tiny_skia::BlendMode::HardLight,
        BlendMode::SoftLight => tiny_skia::BlendMode::SoftLight,
        BlendMode::Difference => tiny_skia::BlendMode::Difference,
        BlendMode::Exclusion => tiny_skia::BlendMode::Exclusion,
    }
}

#[doc = include_str!("skia_lowering.doc/sk_dimensions.md")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn sk_dimensions(size: Size) -> (u32, u32) {
    let width = size.width.round().max(0.0) as u32;
    let height = size.height.round().max(0.0) as u32;
    (width, height)
}

#[doc = include_str!("skia_lowering.doc/to_sk_path.md")]
pub(crate) fn to_sk_path(path: &Path) -> Option<tiny_skia::Path> {
    let mut builder = tiny_skia::PathBuilder::new();
    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(p) => builder.move_to(p.x, p.y),
            PathSegment::LineTo(p) => builder.line_to(p.x, p.y),
            PathSegment::QuadTo { control, end } => {
                builder.quad_to(control.x, control.y, end.x, end.y);
            }
            PathSegment::CubicTo {
                control1,
                control2,
                end,
            } => builder.cubic_to(control1.x, control1.y, control2.x, control2.y, end.x, end.y),
            PathSegment::Close => builder.close(),
        }
    }
    builder.finish()
}

#[doc = include_str!("skia_lowering.doc/to_sk_transform.md")]
pub(crate) fn to_sk_transform(t: &AffineTransform) -> tiny_skia::Transform {
    let m = t.coeffs;
    tiny_skia::Transform::from_row(m[0], m[1], m[2], m[3], m[4], m[5])
}

#[doc = include_str!("skia_lowering.doc/to_sk_fill_rule.md")]
pub(crate) fn to_sk_fill_rule(rule: FillRule) -> tiny_skia::FillRule {
    match rule {
        FillRule::NonZero => tiny_skia::FillRule::Winding,
        FillRule::EvenOdd => tiny_skia::FillRule::EvenOdd,
    }
}

#[doc = include_str!("skia_lowering.doc/to_sk_spread.md")]
pub(crate) fn to_sk_spread(mode: obscur_scene::paint::ExtendMode) -> tiny_skia::SpreadMode {
    match mode {
        obscur_scene::paint::ExtendMode::Pad => tiny_skia::SpreadMode::Pad,
        obscur_scene::paint::ExtendMode::Repeat => tiny_skia::SpreadMode::Repeat,
        obscur_scene::paint::ExtendMode::Reflect => tiny_skia::SpreadMode::Reflect,
    }
}

#[doc = include_str!("skia_lowering.doc/aliased_solid_paint.md")]
pub(crate) fn aliased_solid_paint(color: Color) -> Option<tiny_skia::Paint<'static>> {
    let sk_color = to_sk_color(color)?;
    Some(tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(sk_color),
        anti_alias: false,
        ..Default::default()
    })
}

#[doc = include_str!("skia_lowering.doc/to_sk_color.md")]
pub(crate) fn to_sk_color(color: Color) -> Option<tiny_skia::Color> {
    tiny_skia::Color::from_rgba(
        color.r.clamp(0.0, 1.0),
        color.g.clamp(0.0, 1.0),
        color.b.clamp(0.0, 1.0),
        color.a.clamp(0.0, 1.0),
    )
}

#[doc = include_str!("skia_lowering.doc/solid_paint.md")]
pub(crate) fn solid_paint(color: Color) -> Option<tiny_skia::Paint<'static>> {
    let sk_color = to_sk_color(color)?;
    Some(tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(sk_color),
        anti_alias: true,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_round_to_the_nearest_pixel_and_clamp_negatives() {
        assert_eq!(sk_dimensions(Size::new(16.4, 16.6)), (16, 17));
        assert_eq!(sk_dimensions(Size::new(-3.0, 0.0)), (0, 0));
    }

    #[test]
    fn an_empty_outline_lowers_to_no_path() {
        assert!(to_sk_path(&Path::new()).is_none());
    }

    #[test]
    fn transform_coefficients_keep_their_row_order() {
        let t = AffineTransform {
            coeffs: [2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
        };
        let sk = to_sk_transform(&t);
        assert_eq!(
            (sk.sx, sk.ky, sk.kx, sk.sy, sk.tx, sk.ty),
            (2.0, 3.0, 4.0, 5.0, 6.0, 7.0)
        );
    }

    #[test]
    fn out_of_range_components_clamp_rather_than_fail() {
        let color = to_sk_color(Color::rgba(1.5, -0.5, 0.25, 2.0)).expect("clamped color");
        assert_eq!(
            (color.red(), color.green(), color.blue(), color.alpha()),
            (1.0, 0.0, 0.25, 1.0)
        );
    }

    #[test]
    fn a_conic_wedge_paint_is_aliased_and_a_plain_fill_is_not() {
        let color = Color::rgba(1.0, 0.0, 0.0, 1.0);
        assert!(!aliased_solid_paint(color).expect("paint").anti_alias);
        assert!(solid_paint(color).expect("paint").anti_alias);
    }

    #[test]
    fn the_neutral_blend_mode_is_source_over() {
        assert_eq!(
            to_sk_blend_mode(BlendMode::Normal),
            tiny_skia::BlendMode::SourceOver
        );
        assert_eq!(
            to_sk_blend_mode(BlendMode::Multiply),
            tiny_skia::BlendMode::Multiply
        );
    }
}
