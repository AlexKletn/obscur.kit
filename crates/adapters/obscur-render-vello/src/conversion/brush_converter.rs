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

    /// Converts an obscur-scene [`Stroke`] style into a [`kurbo::Stroke`].
    ///
    /// Width, join, both caps, and the miter limit are carried across; the
    /// stroke's brush is handled separately by the scene converter.
    #[must_use]
    pub fn stroke(&self, stroke: &Stroke) -> kurbo::Stroke {
        kurbo::Stroke::new(f64::from(stroke.width()))
            .with_join(join(stroke.line_join()))
            .with_caps(cap(stroke.line_cap()))
            .with_miter_limit(f64::from(stroke.miter_limit()))
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
}
