#![doc = include_str!("../../docs/api/renderer/brush_fill.doc/_module.md")]

use obscur_scene::color::Color;
use obscur_scene::color::convert::{oklab_lerp, premul_oklab};
use obscur_scene::paint::Brush;

use crate::renderer::clip_mask::build_clip_mask;
use crate::renderer::skia_lowering::{aliased_solid_paint, solid_paint, to_sk_color, to_sk_spread};

/// Returns the representative color for drawing paths that use solid paints. Gradients contribute their first stop, when present.
pub(crate) fn brush_color(brush: &Brush) -> Option<Color> {
    match brush {
        Brush::Solid(solid) => Some(solid.color),
        Brush::Linear(gradient) => gradient.stops().first().map(|stop| stop.color),
        Brush::Radial(gradient) => gradient.stops().first().map(|stop| stop.color),
        Brush::Conic(gradient) => gradient.stops().first().map(|stop| stop.color),
    }
}

#[doc = include_str!("../../docs/api/renderer/brush_fill.doc/fill_conic.md")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn fill_conic(
    pixmap: &mut tiny_skia::Pixmap,
    conic: &obscur_scene::paint::ConicGradient,
    sk_path: &tiny_skia::Path,
    fill_rule: tiny_skia::FillRule,
    width: u32,
    height: u32,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) {
    let stops = conic.stops();
    let Some(&first) = stops.first() else {
        return;
    };
    let last = stops[stops.len() - 1];
    // The element outline (∩ active clip) becomes the mask every wedge is confined
    // to. Without it there is nothing to fill.
    let Some(region) = build_clip_mask(width, height, sk_path, fill_rule, clip, transform) else {
        return;
    };
    let center = conic.center();
    let start = conic.start_angle();
    // A radius that reaches beyond every corner of the buffer from the center, so a
    // wedge's outer edge always lies outside the region being filled.
    #[allow(clippy::cast_precision_loss)]
    let radius = (width as f32 + center.x.abs()).hypot(height as f32 + center.y.abs()) + 1.0;

    // The gap-free segment list covering the whole `[0, 1]` turn: a leading pad, the
    // inter-stop spans, and a trailing pad (each `(o0, c0, o1, c1)`).
    let mut segments: Vec<(f32, Color, f32, Color)> = Vec::new();
    if first.offset() > 0.0 {
        segments.push((0.0, first.color(), first.offset(), first.color()));
    }
    for pair in stops.windows(2) {
        if pair[1].offset() > pair[0].offset() {
            segments.push((
                pair[0].offset(),
                pair[0].color(),
                pair[1].offset(),
                pair[1].color(),
            ));
        }
    }
    if last.offset() < 1.0 {
        segments.push((last.offset(), last.color(), 1.0, last.color()));
    }
    // A single stop (or all-coincident stops) paints the whole region one color.
    if segments.is_empty() {
        segments.push((0.0, first.color(), 1.0, first.color()));
    }

    for (o0, c0, o1, c1) in segments {
        let span = o1 - o0;
        if span <= 0.0 {
            continue;
        }
        // ~1° per sub-wedge so the outer arc stays outside the box and a smooth
        // span reads as a gradient rather than a fan of facets.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let steps = ((span * 360.0).ceil() as usize).max(1);
        let from = premul_oklab(c0);
        let to = premul_oklab(c1);
        for k in 0..steps {
            #[allow(clippy::cast_precision_loss)]
            let f0 = k as f32 / steps as f32;
            #[allow(clippy::cast_precision_loss)]
            let f1 = (k + 1) as f32 / steps as f32;
            let color = oklab_lerp(from, to, f64::from((f0 + f1) * 0.5));
            let Some(sk_paint) = aliased_solid_paint(color) else {
                continue;
            };
            let a0 = start + (o0 + span * f0) * std::f32::consts::TAU;
            let a1 = start + (o0 + span * f1) * std::f32::consts::TAU;
            let mut builder = tiny_skia::PathBuilder::new();
            builder.move_to(center.x, center.y);
            builder.line_to(center.x + radius * a0.cos(), center.y + radius * a0.sin());
            builder.line_to(center.x + radius * a1.cos(), center.y + radius * a1.sin());
            builder.close();
            if let Some(wedge) = builder.finish() {
                pixmap.fill_path(
                    &wedge,
                    &sk_paint,
                    tiny_skia::FillRule::Winding,
                    transform,
                    Some(&region),
                );
            }
        }
    }
}

/// Number of evenly spaced segments used to approximate each Oklab span in the tiny-skia linear shader.
const GRADIENT_SUBDIVISIONS: usize = 32;

#[doc = include_str!("../../docs/api/renderer/brush_fill.doc/fill_linear.md")]
pub(crate) fn fill_linear(
    pixmap: &mut tiny_skia::Pixmap,
    linear: &obscur_scene::paint::LinearGradient,
    sk_path: &tiny_skia::Path,
    fill_rule: tiny_skia::FillRule,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) {
    let stops = linear.stops();
    let sk_stops = gradient_stops(stops);

    // Fewer than two representable stops cannot form a shader: fall back to the flat
    // first-stop fill (the documented solid/stop-less degenerate path).
    let first_stop_paint = || stops.first().map(|s| s.color()).and_then(solid_paint);
    if sk_stops.len() < 2 {
        if let Some(sk_paint) = first_stop_paint() {
            pixmap.fill_path(sk_path, &sk_paint, fill_rule, transform, clip);
        }
        return;
    }

    let start = tiny_skia::Point::from_xy(linear.start().x, linear.start().y);
    let end = tiny_skia::Point::from_xy(linear.end().x, linear.end().y);
    let mode = to_sk_spread(linear.extend());
    let Some(shader) = tiny_skia::LinearGradient::new(
        start,
        end,
        sk_stops,
        mode,
        tiny_skia::Transform::identity(),
    ) else {
        // A degenerate axis (start == end) has no direction: fall back to the flat
        // first-stop fill rather than dropping the mark.
        if let Some(sk_paint) = first_stop_paint() {
            pixmap.fill_path(sk_path, &sk_paint, fill_rule, transform, clip);
        }
        return;
    };
    let sk_paint = tiny_skia::Paint {
        shader,
        anti_alias: true,
        ..Default::default()
    };
    pixmap.fill_path(sk_path, &sk_paint, fill_rule, transform, clip);
}

/// Samples authored gradient stops in premultiplied Oklab for tiny-skia shaders.
fn gradient_stops(stops: &[obscur_scene::paint::GradientStop]) -> Vec<tiny_skia::GradientStop> {
    // Build the dense Oklab-sampled stop list. Each authored span is
    // subdivided, keeping exact authored colors and duplicate stops for
    // hard edges.
    let mut sk_stops: Vec<tiny_skia::GradientStop> = Vec::new();
    let mut push = |offset: f32, color: Color| {
        if let Some(sk_color) = to_sk_color(color) {
            sk_stops.push(tiny_skia::GradientStop::new(
                offset.clamp(0.0, 1.0),
                sk_color,
            ));
        }
    };
    if let Some(first) = stops.first() {
        push(first.offset(), first.color());
    }
    for pair in stops.windows(2) {
        let (s0, s1) = (pair[0], pair[1]);
        let span = s1.offset() - s0.offset();
        if span > 0.0 {
            let (from, to) = (premul_oklab(s0.color()), premul_oklab(s1.color()));
            for k in 1..GRADIENT_SUBDIVISIONS {
                #[allow(clippy::cast_precision_loss)]
                let f = k as f32 / GRADIENT_SUBDIVISIONS as f32;
                push(
                    span.mul_add(f, s0.offset()),
                    oklab_lerp(from, to, f64::from(f)),
                );
            }
        }
        push(s1.offset(), s1.color());
    }

    sk_stops
}

/// Maps a unit radial shader into the authored ellipse, with the first stop as a fallback for degenerate geometry.
pub(crate) fn radial_paint(
    gradient: &obscur_scene::paint::RadialGradient,
) -> Option<tiny_skia::Paint<'static>> {
    let stops = gradient.stops();
    let center = gradient.center();
    let rx = gradient.radius();
    let ry = gradient.radius_y();
    if stops.len() < 2
        || ![rx, ry, center.x, center.y].into_iter().all(f32::is_finite)
        || rx <= 0.0
        || ry <= 0.0
    {
        return stops.first().and_then(|stop| solid_paint(stop.color()));
    }
    let shader = tiny_skia::RadialGradient::new(
        tiny_skia::Point::from_xy(0.0, 0.0),
        tiny_skia::Point::from_xy(0.0, 0.0),
        1.0,
        gradient_stops(stops),
        to_sk_spread(gradient.extend()),
        tiny_skia::Transform::from_row(rx, 0.0, 0.0, ry, center.x, center.y),
    );
    shader.map_or_else(
        || stops.first().and_then(|stop| solid_paint(stop.color())),
        |shader| {
            Some(tiny_skia::Paint {
                shader,
                anti_alias: true,
                ..Default::default()
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Point;

    #[test]
    fn representative_colors_of_radial_and_conic_brushes_use_the_first_stop() {
        use obscur_scene::paint::{ConicGradient, GradientStop, RadialGradient};

        let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
        let blue = Color::rgba(0.0, 0.0, 1.0, 1.0);
        let stops = || vec![GradientStop::new(0.0, red), GradientStop::new(1.0, blue)];
        // Both approximate to the first stop (red), per the effect-split policy.
        let radial = Brush::Radial(RadialGradient::new(Point::new(5.0, 5.0), 10.0, stops()));
        assert_eq!(brush_color(&radial), Some(red));
        let conic = Brush::Conic(ConicGradient::new(Point::new(5.0, 5.0), 0.0, stops()));
        assert_eq!(brush_color(&conic), Some(red));
        // A stop-less gradient resolves to no color (the command is skipped).
        let empty = Brush::Radial(RadialGradient::new(Point::ORIGIN, 1.0, Vec::new()));
        assert_eq!(brush_color(&empty), None);
    }

    #[test]
    fn premultiplied_interpolation_does_not_fade_through_grey() {
        // css-color-4 §12.3: `transparent → blue` stays full-strength blue at the
        // midpoint, at half alpha. The straight lerp is the wrong answer it guards.
        let transparent = Color::rgba(0.0, 0.0, 0.0, 0.0);
        let blue = Color::rgba(0.0, 0.0, 1.0, 1.0);
        let premul = oklab_lerp(premul_oklab(transparent), premul_oklab(blue), 0.5);
        assert!((premul.b - 1.0).abs() < 1e-5, "colour stays full strength");
        assert_eq!(premul.a, 0.5, "alpha carries the fade");
    }

    #[test]
    fn a_fully_transparent_interpolation_is_the_transparent_colour() {
        let clear = Color::rgba(0.0, 0.0, 0.0, 0.0);
        assert_eq!(
            oklab_lerp(premul_oklab(clear), premul_oklab(clear), 0.5),
            clear
        );
    }
}
