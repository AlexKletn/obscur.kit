#![doc = include_str!("progress_painter.doc/_module.md")]

use obscur_geometry::{Point, Size};
use obscur_scene::command::CanvasRegion;
use obscur_scene::paint::{Brush, LineCap, Stroke};
use obscur_scene::path::{Path, PathTrim};
use obscur_view::view::{ViewProperty, ViewValue};

use super::{Progress, ProgressShape};

#[doc = include_str!("progress_painter.doc/THICKNESS.md")]
const THICKNESS: f32 = 6.0;

#[doc = include_str!("progress_painter.doc/paint_progress.md")]
pub fn paint_progress(region: &mut CanvasRegion<'_>, props: &[ViewProperty], accent: Brush) {
    let size = region.size();
    let value = read_value(props);
    let shape = ProgressShape::from_int(read_shape(props));
    let track_path = track_path(shape, size);
    if track_path.is_empty() {
        return;
    }

    let thickness = THICKNESS.min(size.width.min(size.height));
    // Round caps are why this is a real stroked arc and not a gradient
    // pretending to be one: a ring at 3% is a dot with two rounded ends, and no
    // conic gradient can produce that.
    let pen = |brush: Brush| Stroke::new(brush, thickness).with_line_cap(LineCap::Round);

    // The track first, then the filled span over it. Both are the SAME path,
    // which is what keeps the fill exactly on the track however the shape is
    // built — including a shape this widget did not design.
    region
        .painter()
        .stroke(track_path.clone(), pen(track_brush()));

    let filled = PathTrim::of(&track_path).slice(0.0, value);
    if !filled.is_empty() {
        region.painter().stroke(filled, pen(accent));
    }
}

#[doc = include_str!("progress_painter.doc/track_path.md")]
fn track_path(shape: ProgressShape, size: Size) -> Path {
    match shape {
        ProgressShape::Line => {
            let mut path = Path::new();
            if size.width <= 0.0 {
                return path;
            }
            let y = size.height / 2.0;
            path.move_to(Point::new(0.0, y));
            path.line_to(Point::new(size.width, y));
            path
        }
        ProgressShape::Ring => ring(size),
    }
}

#[doc = include_str!("progress_painter.doc/ring.md")]
fn ring(size: Size) -> Path {
    // Four cubics, the standard circle approximation, starting at twelve
    // o'clock and running clockwise — so a ring fills the way a clock does,
    // which is the only reading a user brings to it.
    const K: f32 = 0.552_285;
    let mut path = Path::new();
    let radius = (size.width.min(size.height) / 2.0) - THICKNESS / 2.0;
    if radius <= 0.0 {
        return path;
    }
    let (cx, cy) = (size.width / 2.0, size.height / 2.0);
    let k = radius * K;
    path.move_to(Point::new(cx, cy - radius));
    path.cubic_to(
        Point::new(cx + k, cy - radius),
        Point::new(cx + radius, cy - k),
        Point::new(cx + radius, cy),
    );
    path.cubic_to(
        Point::new(cx + radius, cy + k),
        Point::new(cx + k, cy + radius),
        Point::new(cx, cy + radius),
    );
    path.cubic_to(
        Point::new(cx - k, cy + radius),
        Point::new(cx - radius, cy + k),
        Point::new(cx - radius, cy),
    );
    path.cubic_to(
        Point::new(cx - radius, cy - k),
        Point::new(cx - k, cy - radius),
        Point::new(cx, cy - radius),
    );
    path
}

#[doc = include_str!("progress_painter.doc/track_brush.md")]
fn track_brush() -> Brush {
    // The unfilled remainder, faint. It is drawn by the painter rather than
    // left to the sheet because it must follow the SAME path as the fill: a
    // track styled by CSS could not follow a shape CSS never saw.
    Brush::solid(obscur_scene::color::Color::rgba(0.17, 0.16, 0.15, 0.25))
}

#[doc = include_str!("progress_painter.doc/read_value.md")]
fn read_value(props: &[ViewProperty]) -> f32 {
    props
        .iter()
        .find(|p| p.name() == Progress::PROP_VALUE)
        .and_then(|p| match p.value() {
            // A fraction is a fraction: the precision an `f64` carries beyond an
            // `f32` is far below a pixel of any bar, and the clamp below bounds
            // whatever arrives.
            #[allow(clippy::cast_possible_truncation)]
            ViewValue::Float(v) => Some(*v as f32),
            #[allow(clippy::cast_precision_loss)]
            ViewValue::Int(v) => Some(*v as f32),
            _ => None,
        })
        .map_or(0.0, |v| {
            if v.is_finite() {
                v.clamp(0.0, 1.0)
            } else {
                0.0
            }
        })
}

#[doc = include_str!("progress_painter.doc/read_shape.md")]
fn read_shape(props: &[ViewProperty]) -> i64 {
    props
        .iter()
        .find(|p| p.name() == Progress::PROP_SHAPE)
        .and_then(|p| match p.value() {
            ViewValue::Int(v) => Some(*v),
            _ => None,
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{paint_progress, ring, track_path};
    use crate::progress::{Progress, ProgressShape};
    use obscur_geometry::{Point, Rect, Size};
    use obscur_scene::color::Color;
    use obscur_scene::command::{CanvasRegion, DisplayListBuilder, PaintCommand};
    use obscur_scene::paint::{Brush, LineCap};
    use obscur_scene::path::PathTrim;
    use obscur_view::view::{ViewProperty, ViewValue};

    const BOX: Size = Size {
        width: 100.0,
        height: 100.0,
    };

    fn props(value: f64, shape: ProgressShape) -> Vec<ViewProperty> {
        vec![
            ViewProperty::new(Progress::PROP_VALUE, ViewValue::Float(value)),
            ViewProperty::new(Progress::PROP_SHAPE, ViewValue::Int(shape.as_int())),
        ]
    }

    fn accent() -> Brush {
        Brush::solid(Color::rgba(0.2, 0.5, 1.0, 1.0))
    }

    /// The strokes a paint produced, as `(command count, widths)`.
    fn strokes(value: f64, shape: ProgressShape) -> Vec<(f32, LineCap)> {
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, Rect::new(Point::new(0.0, 0.0), BOX));
        paint_progress(&mut region, &props(value, shape), accent());
        let _ = region.close();
        builder
            .build()
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath {
                    paint: obscur_scene::paint::Paint::Stroke(stroke),
                    ..
                } => Some((stroke.width(), stroke.line_cap())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_bar_draws_its_track_and_its_fill_over_it() {
        let at_half = strokes(0.5, ProgressShape::Line);
        assert_eq!(at_half.len(), 2, "the track, then the filled span");
    }

    #[test]
    fn an_empty_bar_draws_the_track_and_nothing_else() {
        // The fill is a slice of length zero, and a zero-length slice is empty
        // rather than a dot: a bar at 0% must look unstarted.
        assert_eq!(strokes(0.0, ProgressShape::Line).len(), 1);
    }

    #[test]
    fn both_ends_are_round_which_is_the_whole_reason_this_is_a_stroked_path() {
        // A conic gradient cannot produce a rounded cap. This assertion is what
        // says the ring is a real arc rather than an imitation of one.
        for (_, cap) in strokes(0.4, ProgressShape::Ring) {
            assert_eq!(cap, LineCap::Round);
        }
    }

    #[test]
    fn the_fill_is_a_trim_of_the_very_path_the_track_is() {
        // Track and fill share one path, so a fill can never drift off its own
        // track — including along a shape this widget did not design.
        let track = track_path(ProgressShape::Ring, BOX);
        let measured = PathTrim::of(&track);
        let quarter = measured.slice(0.0, 0.25);
        assert!(
            !quarter.is_empty(),
            "a quarter of a ring is a real arc, not nothing"
        );
        let full = measured.length();
        let cut = PathTrim::of(&quarter).length();
        assert!(
            (cut - full / 4.0).abs() < full * 0.02,
            "a quarter measures a quarter: {cut} of {full}"
        );
    }

    #[test]
    fn a_ring_is_round_and_starts_at_twelve_oclock() {
        let path = ring(BOX);
        let bounds = path.bounds().expect("a ring has bounds");
        assert!(
            (bounds.size.width - bounds.size.height).abs() < 0.01,
            "a ring is as wide as it is tall: {bounds:?}"
        );
        // The first point is the top of the circle — a ring fills the way a
        // clock runs, which is the only reading a user brings to it.
        let first = path
            .segments()
            .iter()
            .find_map(|s| match s {
                obscur_scene::path::PathSegment::MoveTo(p) => Some(*p),
                _ => None,
            })
            .expect("a ring starts somewhere");
        assert!((first.x - BOX.width / 2.0).abs() < 0.01, "{first:?}");
        assert!(first.y < BOX.height / 2.0, "above the centre: {first:?}");
    }

    #[test]
    fn a_box_too_small_for_the_shape_draws_nothing_rather_than_a_smear() {
        let tiny = Size::new(1.0, 1.0);
        assert!(track_path(ProgressShape::Ring, tiny).is_empty());
        assert!(track_path(ProgressShape::Line, Size::new(0.0, 10.0)).is_empty());
    }

    #[test]
    fn a_node_carrying_no_properties_paints_an_empty_line() {
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, Rect::new(Point::new(0.0, 0.0), BOX));
        paint_progress(&mut region, &[], accent());
        assert!(region.close(), "and leaves the region balanced");
    }
}
