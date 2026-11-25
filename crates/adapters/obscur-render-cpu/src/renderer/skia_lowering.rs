#![doc = include_str!("../../docs/api/renderer/skia_lowering.doc/_module.md")]

use obscur_geometry::{AffineTransform, Size};
use obscur_scene::color::Color;
use obscur_scene::paint::BlendMode;
use obscur_scene::path::{FillRule, Path, PathSegment};

/// Maps a scene [`BlendMode`] onto the `tiny-skia` compositing operator. The
/// separable CSS `mix-blend-mode` values map one-to-one; [`BlendMode::Normal`] is
/// `SourceOver`, the neutral operator.
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
        BlendMode::Hue => tiny_skia::BlendMode::Hue,
        BlendMode::Saturation => tiny_skia::BlendMode::Saturation,
        BlendMode::Color => tiny_skia::BlendMode::Color,
        BlendMode::Luminosity => tiny_skia::BlendMode::Luminosity,
    }
}

/// Computes the integer pixel dimensions of a target from its logical [`Size`],
/// rounding to the nearest pixel and clamping finite negatives to zero.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn sk_dimensions(size: Size) -> Option<(u32, u32)> {
    let dimension = |value: f32| {
        let rounded = f64::from(value).round();
        (rounded.is_finite() && rounded <= f64::from(u32::MAX)).then(|| rounded.max(0.0) as u32)
    };
    dimension(size.width).zip(dimension(size.height))
}

/// Converts a domain [`Path`] into a [`tiny_skia::Path`], or `None` when the
/// outline is empty or degenerate (`tiny_skia::PathBuilder::finish` rejects it).
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

/// Converts an obscur-geometry [`AffineTransform`] into a
/// [`tiny_skia::Transform`]. The obscur coefficients `[a, b, c, d, tx, ty]` map to
/// tiny-skia's row order `(sx, ky, kx, sy, tx, ty)` one-to-one (both map a point
/// `(x, y)` to `(a·x + c·y + tx, b·x + d·y + ty)`).
pub(crate) fn to_sk_transform(t: &AffineTransform) -> tiny_skia::Transform {
    let m = t.coeffs;
    tiny_skia::Transform::from_row(m[0], m[1], m[2], m[3], m[4], m[5])
}

/// Maps a domain [`FillRule`] onto the `tiny-skia` equivalent.
pub(crate) fn to_sk_fill_rule(rule: FillRule) -> tiny_skia::FillRule {
    match rule {
        FillRule::NonZero => tiny_skia::FillRule::Winding,
        FillRule::EvenOdd => tiny_skia::FillRule::EvenOdd,
    }
}

/// Maps a domain [`ExtendMode`](obscur_scene::paint::ExtendMode) onto tiny-skia's
/// [`SpreadMode`](tiny_skia::SpreadMode) for gradient sampling outside `[0, 1]`.
pub(crate) fn to_sk_spread(mode: obscur_scene::paint::ExtendMode) -> tiny_skia::SpreadMode {
    match mode {
        obscur_scene::paint::ExtendMode::Pad => tiny_skia::SpreadMode::Pad,
        obscur_scene::paint::ExtendMode::Repeat => tiny_skia::SpreadMode::Repeat,
        obscur_scene::paint::ExtendMode::Reflect => tiny_skia::SpreadMode::Reflect,
    }
}

/// An **aliased** solid-color [`tiny_skia::Paint`] — used for conic-gradient wedges,
/// whose shared internal edges must tile pixel-exactly without the AA seam an
/// anti-aliased fill would leave between adjacent triangles.
pub(crate) fn aliased_solid_paint(color: Color) -> Option<tiny_skia::Paint<'static>> {
    let sk_color = to_sk_color(color)?;
    Some(tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(sk_color),
        anti_alias: false,
        ..Default::default()
    })
}

/// Maps a domain [`Color`] (straight-alpha sRGB, `f32`) onto a
/// [`tiny_skia::Color`], clamping each component to `0.0..=1.0`. Returns `None`
/// only if `tiny-skia` rejects the clamped components.
pub(crate) fn to_sk_color(color: Color) -> Option<tiny_skia::Color> {
    tiny_skia::Color::from_rgba(
        color.r.clamp(0.0, 1.0),
        color.g.clamp(0.0, 1.0),
        color.b.clamp(0.0, 1.0),
        color.a.clamp(0.0, 1.0),
    )
}

/// Builds an anti-aliased solid-color [`tiny_skia::Paint`] from a domain
/// [`Color`], or `None` when the color cannot be represented.
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
        assert_eq!(sk_dimensions(Size::new(16.4, 16.6)), Some((16, 17)));
        assert_eq!(sk_dimensions(Size::new(-3.0, 0.0)), Some((0, 0)));
        assert_eq!(sk_dimensions(Size::new(f32::NAN, 1.0)), None);
        assert_eq!(sk_dimensions(Size::new(4_294_967_296.0, 1.0)), None);
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
