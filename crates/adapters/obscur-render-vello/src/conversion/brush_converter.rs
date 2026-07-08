//! Converts obscur-scene paint sources ([`Brush`], [`Stroke`]) into peniko/kurbo.
//!
//! Pure, backend-local conversion (spec §2.7). Solid and gradient brushes map to
//! [`peniko::Brush`]; stroke style maps to a [`kurbo::Stroke`]. Stop colors and
//! offsets, and the stroke's width/join/cap, are preserved exactly (spec §5.2 —
//! the adapter keeps paint data authoritative and delegates only rasterization).

use obscur_scene::paint::{Brush, LineCap, LineJoin, Stroke};
use vello::kurbo;
use vello::peniko;

use super::color_converter::ColorConverter;

/// A converter from obscur-scene paint sources to peniko/kurbo equivalents.
///
/// It owns a [`ColorConverter`] so brush and gradient-stop colors go through the
/// same sRGB mapping.
///
/// # Gradient mapping
///
/// - [`Brush::Linear`] → [`peniko::Gradient::new_linear`] over the same axis.
/// - [`Brush::Radial`] → [`peniko::Gradient::new_radial`] over the same
///   center/radius. The domain models a concentric radial gradient, which is the
///   single-circle case of peniko's radial gradient.
/// - [`Brush::Conic`] → [`peniko::Gradient::new_sweep`]. The domain conic
///   gradient sweeps one full turn clockwise from `start_angle`, so it maps to a
///   sweep over `[start_angle, start_angle + 2π]`. peniko's sweep advances in the
///   same offset direction, so the stop offsets and colors are preserved; only
///   the exact angular rasterization is Vello's (spec §5.2).
///
/// In every case the gradient's color stops (offset + converted color) and its
/// [extend mode](obscur_scene::paint::ExtendMode) carry over unchanged.
#[derive(Debug, Default, Clone, Copy)]
pub struct BrushConverter {
    color: ColorConverter,
}

impl BrushConverter {
    /// Creates a brush converter using the given color converter.
    #[must_use]
    pub const fn new(color: ColorConverter) -> Self {
        Self { color }
    }

    /// Converts an obscur-scene [`Brush`] into a [`peniko::Brush`].
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
                .with_stops(self.stops(g.stops()).as_slice());
                peniko::Brush::Gradient(gradient)
            }
            Brush::Radial(g) => {
                let gradient = peniko::Gradient::new_radial(
                    (f64::from(g.center().x), f64::from(g.center().y)),
                    g.radius(),
                )
                .with_extend(extend(g.extend()))
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
                .with_stops(self.stops(g.stops()).as_slice());
                peniko::Brush::Gradient(gradient)
            }
        }
    }

    /// The optional brush transform to pass alongside [`convert`](Self::convert)
    /// when filling/stroking with this brush.
    ///
    /// A [`Brush::Radial`] with an **elliptical** ending shape is emitted by
    /// [`convert`](Self::convert) as a concentric *circle* gradient of the x-axis
    /// radius; this transform stretches that circle into the authored ellipse by
    /// scaling the y-axis by `radius_y / radius` about the gradient center (spec
    /// §3.2). Every other brush (solid, linear, conic, or a circular radial) needs
    /// no brush transform and yields [`None`].
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

    /// Converts an obscur-scene [`Stroke`] style into a [`kurbo::Stroke`].
    ///
    /// Width, join, both caps, and the miter limit are carried across; the
    /// stroke's brush is handled separately by the scene converter. A non-empty
    /// [`Stroke::dash`] is mapped onto kurbo's dash pattern (widened to `f64`)
    /// with its phase offset, so a dashed scene stroke rasterizes as dashes; an
    /// empty dash leaves kurbo's default (solid) dash, byte-identical to before.
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

    /// Converts obscur-scene gradient stops into peniko color stops.
    fn stops(self, stops: &[obscur_scene::paint::GradientStop]) -> Vec<peniko::ColorStop> {
        stops
            .iter()
            .map(|stop| peniko::ColorStop::from((stop.offset(), self.color.convert(stop.color()))))
            .collect()
    }
}

/// Maps an obscur-scene extend mode to a peniko extend mode.
fn extend(mode: obscur_scene::paint::ExtendMode) -> peniko::Extend {
    match mode {
        obscur_scene::paint::ExtendMode::Pad => peniko::Extend::Pad,
        obscur_scene::paint::ExtendMode::Repeat => peniko::Extend::Repeat,
        obscur_scene::paint::ExtendMode::Reflect => peniko::Extend::Reflect,
    }
}

/// Maps an obscur-scene line join to a kurbo join.
fn join(join: LineJoin) -> kurbo::Join {
    match join {
        LineJoin::Miter => kurbo::Join::Miter,
        LineJoin::Round => kurbo::Join::Round,
        LineJoin::Bevel => kurbo::Join::Bevel,
    }
}

/// Maps an obscur-scene line cap to a kurbo cap.
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
                assert_eq!(gradient.stops.len(), 2);
                assert_eq!(gradient.extend, peniko::Extend::Pad);
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
                assert_eq!(gradient.stops.len(), 2);
                assert!(matches!(gradient.kind, peniko::GradientKind::Radial(_)));
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
        assert!((c.x - 5.0).abs() < 1e-9 && (c.y - 5.0).abs() < 1e-9, "center fixed: {c:?}");
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
                assert_eq!(gradient.stops.len(), 2);
                assert!(matches!(gradient.kind, peniko::GradientKind::Sweep(_)));
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
