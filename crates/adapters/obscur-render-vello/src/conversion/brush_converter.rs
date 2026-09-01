#![doc = include_str!("brush_converter.doc/_module.md")]

use obscur_scene::color::Color;
use obscur_scene::color::convert::{linear_srgb_to_oklab, oklab_to_srgb, srgb_decode};
use obscur_scene::paint::{Brush, LineCap, LineJoin, Stroke};
use vello::kurbo;
use vello::peniko;

use super::color_converter::ColorConverter;

#[doc = include_str!("brush_converter.doc/GRADIENT_INTERPOLATION_CS.md")]
const GRADIENT_INTERPOLATION_CS: peniko::color::ColorSpaceTag = peniko::color::ColorSpaceTag::Oklab;

#[doc = include_str!("brush_converter.doc/DENSIFY_SEGMENTS.md")]
const DENSIFY_SEGMENTS: usize = 16;

#[doc = include_str!("brush_converter.doc/BrushConverter.md")]
#[derive(Debug, Default, Clone, Copy)]
pub struct BrushConverter {
    color: ColorConverter,
}

impl BrushConverter {
    #[doc = include_str!("brush_converter.doc/new.md")]
    #[must_use]
    pub const fn new(color: ColorConverter) -> Self {
        Self { color }
    }

    #[doc = include_str!("brush_converter.doc/convert.md")]
    #[must_use]
    pub fn convert(&self, brush: &Brush) -> peniko::Brush {
        match brush {
            Brush::Solid(solid) => peniko::Brush::Solid(self.color.convert(solid.color())),
            Brush::Linear(g) => {
                let gradient = peniko::Gradient::new_linear(
                    (f64::from(g.start().x), f64::from(g.start().y)),
                    (f64::from(g.end().x), f64::from(g.end().y)),
                )
                .with_extend(extend(g.extend()))
                .with_interpolation_cs(GRADIENT_INTERPOLATION_CS)
                .with_stops(self.stops(g.stops()).as_slice());
                peniko::Brush::Gradient(gradient)
            }
            Brush::Radial(g) => {
                let gradient = peniko::Gradient::new_radial(
                    (f64::from(g.center().x), f64::from(g.center().y)),
                    g.radius(),
                )
                .with_extend(extend(g.extend()))
                .with_interpolation_cs(GRADIENT_INTERPOLATION_CS)
                .with_stops(self.stops(g.stops()).as_slice());
                peniko::Brush::Gradient(gradient)
            }
            Brush::Conic(g) => {
                let start = g.start_angle();
                let gradient = peniko::Gradient::new_sweep(
                    (f64::from(g.center().x), f64::from(g.center().y)),
                    start,
                    start + std::f32::consts::TAU,
                )
                .with_extend(extend(g.extend()))
                .with_interpolation_cs(GRADIENT_INTERPOLATION_CS)
                .with_stops(self.stops(g.stops()).as_slice());
                peniko::Brush::Gradient(gradient)
            }
        }
    }

    #[doc = include_str!("brush_converter.doc/brush_transform.md")]
    #[must_use]
    pub fn brush_transform(brush: &Brush) -> Option<kurbo::Affine> {
        match brush {
            Brush::Radial(g) if g.is_elliptical() && g.radius() != 0.0 => {
                let (cx, cy) = (f64::from(g.center().x), f64::from(g.center().y));
                let scale_y = f64::from(g.radius_y()) / f64::from(g.radius());
                Some(
                    kurbo::Affine::translate((cx, cy))
                        * kurbo::Affine::scale_non_uniform(1.0, scale_y)
                        * kurbo::Affine::translate((-cx, -cy)),
                )
            }
            _ => None,
        }
    }

    #[doc = include_str!("brush_converter.doc/stroke.md")]
    #[must_use]
    pub fn stroke(&self, stroke: &Stroke) -> kurbo::Stroke {
        let k = kurbo::Stroke::new(f64::from(stroke.width()))
            .with_join(join(stroke.line_join()))
            .with_caps(cap(stroke.line_cap()))
            .with_miter_limit(f64::from(stroke.miter_limit()));
        if stroke.is_dashed() {
            k.with_dashes(
                f64::from(stroke.dash_offset()),
                stroke.dash().iter().map(|d| f64::from(*d)),
            )
        } else {
            k
        }
    }

    #[doc = include_str!("brush_converter.doc/stops.md")]
    fn stops(self, stops: &[obscur_scene::paint::GradientStop]) -> Vec<peniko::ColorStop> {
        if stops.len() < 2 {
            return stops
                .iter()
                .map(|s| peniko::ColorStop::from((s.offset(), self.color.convert(s.color()))))
                .collect();
        }

        let mut out = Vec::with_capacity((stops.len() - 1) * DENSIFY_SEGMENTS + 1);
        // Exact authored first stop.
        out.push(peniko::ColorStop::from((
            stops[0].offset(),
            self.color.convert(stops[0].color()),
        )));
        for pair in stops.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            // Converted ONCE per authored pair rather than once per sample. Each
            // conversion is three `powf(2.4)` and three `cbrt`, and both ends are
            // invariant across the fifteen interior samples below.
            let (from, to) = (premul_oklab(a.color()), premul_oklab(b.color()));
            // Interior Oklab-interpolated stops for this pair.
            for k in 1..DENSIFY_SEGMENTS {
                let t = k as f64 / DENSIFY_SEGMENTS as f64;
                #[allow(clippy::cast_possible_truncation)]
                let offset = a.offset() + (b.offset() - a.offset()) * t as f32;
                out.push(peniko::ColorStop::from((offset, oklab_lerp(from, to, t))));
            }
            // Exact authored end stop of this pair.
            out.push(peniko::ColorStop::from((
                b.offset(),
                self.color.convert(b.color()),
            )));
        }
        out
    }
}

#[doc = include_str!("brush_converter.doc/premul_oklab.md")]
fn premul_oklab(color: Color) -> [f64; 4] {
    let alpha = f64::from(color.a).clamp(0.0, 1.0);
    let lin = |channel: f32| srgb_decode(f64::from(channel).clamp(0.0, 1.0));
    let [l, a, b] = linear_srgb_to_oklab(lin(color.r), lin(color.g), lin(color.b));
    [l * alpha, a * alpha, b * alpha, alpha]
}

#[doc = include_str!("brush_converter.doc/oklab_lerp.md")]
// The Oklab component names (L/a/b) are the canonical single-letter names from
// CSS Color 4 §9; renaming them would obscure the 1:1 mapping to the spec.
#[allow(clippy::many_single_char_names)]
fn oklab_lerp(pa: [f64; 4], pb: [f64; 4], t: f64) -> peniko::Color {
    let lerp = |x: f64, y: f64| x + (y - x) * t;
    let l = lerp(pa[0], pb[0]);
    let aa = lerp(pa[1], pb[1]);
    let bb = lerp(pa[2], pb[2]);
    let alpha = lerp(pa[3], pb[3]);

    let [r, g, blue] = if alpha > 0.0 {
        oklab_to_srgb(l / alpha, aa / alpha, bb / alpha)
    } else {
        [0.0, 0.0, 0.0]
    };
    #[allow(clippy::cast_possible_truncation)]
    peniko::Color::new([r as f32, g as f32, blue as f32, alpha as f32])
}

#[doc = include_str!("brush_converter.doc/extend.md")]
fn extend(mode: obscur_scene::paint::ExtendMode) -> peniko::Extend {
    match mode {
        obscur_scene::paint::ExtendMode::Pad => peniko::Extend::Pad,
        obscur_scene::paint::ExtendMode::Repeat => peniko::Extend::Repeat,
        obscur_scene::paint::ExtendMode::Reflect => peniko::Extend::Reflect,
    }
}

#[doc = include_str!("brush_converter.doc/join.md")]
fn join(join: LineJoin) -> kurbo::Join {
    match join {
        LineJoin::Miter => kurbo::Join::Miter,
        LineJoin::Round => kurbo::Join::Round,
        LineJoin::Bevel => kurbo::Join::Bevel,
    }
}

#[doc = include_str!("brush_converter.doc/cap.md")]
fn cap(cap: LineCap) -> kurbo::Cap {
    match cap {
        LineCap::Butt => kurbo::Cap::Butt,
        LineCap::Round => kurbo::Cap::Round,
        LineCap::Square => kurbo::Cap::Square,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Point;
    use obscur_scene::color::Color;
    use obscur_scene::paint::{GradientStop, LinearGradient};

    #[test]
    fn solid_brush_maps_to_peniko_solid_with_right_color() {
        let brush = Brush::solid(Color::rgba(1.0, 0.0, 0.0, 1.0));
        match BrushConverter::default().convert(&brush) {
            peniko::Brush::Solid(color) => {
                let rgba8 = color.to_rgba8();
                assert_eq!((rgba8.r, rgba8.g, rgba8.b, rgba8.a), (255, 0, 0, 255));
            }
            other => panic!("expected Solid, got {other:?}"),
        }
    }

    #[test]
    fn linear_gradient_maps_stops_and_extend() {
        let g = LinearGradient::new(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            vec![
                GradientStop::new(0.0, Color::BLACK),
                GradientStop::new(1.0, Color::WHITE),
            ],
        );
        match BrushConverter::default().convert(&Brush::Linear(g)) {
            peniko::Brush::Gradient(gradient) => {
                // Two authored stops densify to 1 + (DENSIFY_SEGMENTS) = 17 stops
                // (exact first + 15 interior Oklab stops + exact end).
                assert_eq!(gradient.stops.len(), DENSIFY_SEGMENTS + 1);
                assert_eq!(gradient.extend, peniko::Extend::Pad);
                // CSS Images 4 default interpolation space is Oklab.
                assert_eq!(
                    gradient.interpolation_cs,
                    peniko::color::ColorSpaceTag::Oklab
                );
                // Endpoints stay EXACT: first stop black, last stop white.
                let first = gradient
                    .stops
                    .first()
                    .expect("first stop")
                    .color
                    .to_alpha_color::<peniko::color::Srgb>()
                    .to_rgba8();
                let last = gradient
                    .stops
                    .last()
                    .expect("last stop")
                    .color
                    .to_alpha_color::<peniko::color::Srgb>()
                    .to_rgba8();
                assert_eq!((first.r, first.g, first.b), (0, 0, 0));
                assert_eq!((last.r, last.g, last.b), (255, 255, 255));
                assert_eq!(gradient.stops.first().unwrap().offset, 0.0);
                assert_eq!(gradient.stops.last().unwrap().offset, 1.0);
            }
            other => panic!("expected Gradient, got {other:?}"),
        }
    }

    #[test]
    fn radial_gradient_maps_to_peniko_gradient() {
        use obscur_scene::paint::RadialGradient;
        let g = RadialGradient::new(
            Point::new(50.0, 25.0),
            40.0,
            vec![
                GradientStop::new(0.0, Color::rgba(1.0, 0.0, 0.0, 1.0)),
                GradientStop::new(1.0, Color::rgba(0.0, 0.0, 1.0, 1.0)),
            ],
        );
        match BrushConverter::default().convert(&Brush::Radial(g)) {
            peniko::Brush::Gradient(gradient) => {
                assert_eq!(gradient.stops.len(), DENSIFY_SEGMENTS + 1);
                assert!(matches!(gradient.kind, peniko::GradientKind::Radial(_)));
                assert_eq!(
                    gradient.interpolation_cs,
                    peniko::color::ColorSpaceTag::Oklab
                );
            }
            other => panic!("expected Gradient, got {other:?}"),
        }
    }

    #[test]
    fn circular_radial_needs_no_brush_transform() {
        use obscur_scene::paint::RadialGradient;
        let g = Brush::Radial(RadialGradient::new(Point::new(5.0, 5.0), 10.0, Vec::new()));
        assert!(BrushConverter::brush_transform(&g).is_none());
    }

    #[test]
    fn elliptical_radial_brush_transform_stretches_y() {
        use obscur_scene::paint::RadialGradient;
        // rx=30, ry=10 about center (5, 5): y scales by ry/rx = 1/3 about the center.
        let g = Brush::Radial(RadialGradient::new_elliptical(
            Point::new(5.0, 5.0),
            30.0,
            10.0,
            Vec::new(),
        ));
        let t = BrushConverter::brush_transform(&g).expect("ellipse carries a brush transform");
        // Center is a fixed point of the scale-about-center transform.
        let c = t * kurbo::Point::new(5.0, 5.0);
        assert!(
            (c.x - 5.0).abs() < 1e-9 && (c.y - 5.0).abs() < 1e-9,
            "center fixed: {c:?}"
        );
        // A point one x-radius right stays put (x unscaled); one x-radius "down"
        // maps to ry away (y scaled by 1/3): 5 + 30/3 = 15.
        let down = t * kurbo::Point::new(5.0, 35.0);
        assert!((down.y - 15.0).abs() < 1e-9, "y stretched to ry: {down:?}");
        let right = t * kurbo::Point::new(35.0, 5.0);
        assert!((right.x - 35.0).abs() < 1e-9, "x unscaled: {right:?}");
    }

    #[test]
    fn conic_gradient_maps_to_peniko_sweep() {
        use obscur_scene::paint::ConicGradient;
        let g = ConicGradient::new(
            Point::new(50.0, 25.0),
            0.0,
            vec![
                GradientStop::new(0.0, Color::rgba(1.0, 0.0, 0.0, 1.0)),
                GradientStop::new(1.0, Color::rgba(0.0, 0.0, 1.0, 1.0)),
            ],
        );
        match BrushConverter::default().convert(&Brush::Conic(g)) {
            peniko::Brush::Gradient(gradient) => {
                assert_eq!(gradient.stops.len(), DENSIFY_SEGMENTS + 1);
                assert!(matches!(gradient.kind, peniko::GradientKind::Sweep(_)));
                assert_eq!(
                    gradient.interpolation_cs,
                    peniko::color::ColorSpaceTag::Oklab
                );
            }
            other => panic!("expected Gradient, got {other:?}"),
        }
    }

    #[test]
    fn oklab_midpoint_of_red_to_green_is_vivid_not_muddy() {
        // CSS red (255,0,0) → CSS green (0,128,0). peniko's default sRGB-gamma
        // midpoint is the naive lerp of the gamma bytes: (127, 64, 0) — a dark muddy
        // olive. Oklab interpolation lifts BOTH channels toward a brighter, more
        // vivid warm orange (≈ (166, 105, 0)); the correctness win is the higher
        // green channel and higher total luminance, not a hue flip.
        let mid = oklab_lerp(
            premul_oklab(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            premul_oklab(Color::rgba(0.0, 0.501_960_8, 0.0, 1.0)),
            0.5,
        );
        let rgba8 = mid.to_rgba8();
        // Naive sRGB-gamma midpoint for reference.
        let (srgb_r, srgb_g): (u16, u16) = (127, 64);
        assert!(
            u16::from(rgba8.g) > srgb_g + 20,
            "Oklab midpoint green {} must clear the muddy sRGB midpoint ({}) by a wide margin",
            rgba8.g,
            srgb_g,
        );
        assert!(
            u16::from(rgba8.r) + u16::from(rgba8.g) + u16::from(rgba8.b) > srgb_r + srgb_g,
            "Oklab midpoint {rgba8:?} must be brighter than the sRGB midpoint (127,64,0)",
        );
    }

    #[test]
    fn densified_stops_preserve_exact_endpoints_and_are_monotonic() {
        let g = LinearGradient::new(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            vec![
                GradientStop::new(0.0, Color::rgba(1.0, 0.0, 0.0, 1.0)),
                GradientStop::new(1.0, Color::rgba(0.0, 0.5, 0.0, 1.0)),
            ],
        );
        match BrushConverter::default().convert(&Brush::Linear(g)) {
            peniko::Brush::Gradient(gradient) => {
                // Offsets are non-decreasing across the dense list.
                let offsets: Vec<f32> = gradient.stops.iter().map(|s| s.offset).collect();
                assert!(
                    offsets.windows(2).all(|w| w[0] <= w[1]),
                    "monotonic: {offsets:?}"
                );
                // Exact authored endpoints.
                let first = gradient
                    .stops
                    .first()
                    .unwrap()
                    .color
                    .to_alpha_color::<peniko::color::Srgb>()
                    .to_rgba8();
                let last = gradient
                    .stops
                    .last()
                    .unwrap()
                    .color
                    .to_alpha_color::<peniko::color::Srgb>()
                    .to_rgba8();
                assert_eq!((first.r, first.g, first.b), (255, 0, 0));
                assert_eq!((last.r, last.g), (0, 128));
            }
            other => panic!("expected Gradient, got {other:?}"),
        }
    }

    #[test]
    fn stroke_preserves_width_and_style() {
        let stroke = Stroke::new(Brush::solid(Color::BLACK), 3.5)
            .with_line_cap(LineCap::Round)
            .with_line_join(LineJoin::Bevel);
        let k = BrushConverter::default().stroke(&stroke);
        assert_eq!(k.width, 3.5);
        assert_eq!(k.start_cap, kurbo::Cap::Round);
        assert_eq!(k.end_cap, kurbo::Cap::Round);
        assert_eq!(k.join, kurbo::Join::Bevel);
        assert_eq!(k.miter_limit, 4.0);
    }

    #[test]
    fn solid_stroke_has_no_dash_pattern() {
        let k = BrushConverter::default().stroke(&Stroke::new(Brush::solid(Color::BLACK), 2.0));
        assert!(k.dash_pattern.is_empty());
        assert_eq!(k.dash_offset, 0.0);
    }

    #[test]
    fn dashed_stroke_maps_to_kurbo_dash_pattern() {
        let stroke = Stroke::new(Brush::solid(Color::BLACK), 2.0).with_dash([4.0, 4.0], 1.0);
        let k = BrushConverter::default().stroke(&stroke);
        assert_eq!(k.dash_pattern.as_slice(), &[4.0f64, 4.0]);
        assert_eq!(k.dash_offset, 1.0);
    }
}
