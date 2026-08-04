#![doc = include_str!("progress_painter.doc/_module.md")]

use obscur_geometry::{Point, Size};
use obscur_scene::command::CanvasRegion;
use obscur_scene::paint::{Brush, LineCap, Stroke};
use obscur_scene::path::{Path, PathTrim};
use obscur_view::view::{CanvasPaint, ViewProperty, ViewValue};

use super::{Progress, ProgressShape};

#[doc = include_str!("progress_painter.doc/THICKNESS.md")]
const THICKNESS: f32 = 6.0;

#[doc = include_str!("progress_painter.doc/SWEEP_SECONDS.md")]
const SWEEP_SECONDS: f32 = 1.4;

#[doc = include_str!("progress_painter.doc/SWEEP_FRACTION.md")]
const SWEEP_FRACTION: f32 = 0.25;

#[doc = include_str!("progress_painter.doc/paint_progress.md")]
pub fn paint_progress(
    region: &mut CanvasRegion<'_>,
    paint: &CanvasPaint<'_>,
    accent: Brush,
) -> bool {
    let props = paint.props();
    let size = region.size();
    let tracked = read_tracked(props);
    let shape = ProgressShape::from_int(read_shape(props));
    let track_path = track_path(shape, size);
    if track_path.is_empty() {
        return false;
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

    let measured = PathTrim::of(&track_path);
    // A tracked bar fills from the start to its value. An untracked one sweeps a
    // window along the track instead — it has no value to show, and a bar frozen
    // at zero would say "nothing is happening" when the truth is "I do not know
    // how much has".
    let (from, to) = if tracked {
        (0.0, read_value(props))
    } else {
        sweep(paint.elapsed())
    };
    let filled = measured.slice(from, to);
    if !filled.is_empty() {
        region.painter().stroke(filled, pen(accent));
    }
    // The whole reason a painter votes: only an untracked bar needs the next
    // frame. A tracked one is redrawn when its value changes, like anything else.
    !tracked
}

#[doc = include_str!("progress_painter.doc/sweep.md")]
fn sweep(elapsed: f32) -> (f32, f32) {
    // The window's LEADING edge runs the whole track and wraps; the trailing
    // edge follows a fixed distance behind. Clamping rather than wrapping the
    // span is deliberate: a window that wrapped would appear at both ends at
    // once, which reads as two bars rather than one travelling.
    let phase = (elapsed / SWEEP_SECONDS).fract();
    let head = phase * (1.0 + SWEEP_FRACTION);
    let tail = head - SWEEP_FRACTION;
    (tail.clamp(0.0, 1.0), head.clamp(0.0, 1.0))
}

#[doc = include_str!("progress_painter.doc/read_tracked.md")]
fn read_tracked(props: &[ViewProperty]) -> bool {
    props
        .iter()
        .find(|p| p.name() == Progress::PROP_TRACKED)
        .and_then(|p| match p.value() {
            ViewValue::Bool(v) => Some(*v),
            _ => None,
        })
        // A node that says nothing is tracked: a bar showing a fraction it was
        // given is the safer default than one that animates forever.
        .unwrap_or(true)
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
    use obscur_view::view::{CanvasPaint, ViewProperty, ViewValue};

    const BOX: Size = Size {
        width: 100.0,
        height: 100.0,
    };

    fn props(value: f64, shape: ProgressShape) -> Vec<ViewProperty> {
        vec![
            ViewProperty::new(Progress::PROP_VALUE, ViewValue::Float(value)),
            ViewProperty::new(Progress::PROP_SHAPE, ViewValue::Int(shape.as_int())),
            ViewProperty::new(Progress::PROP_TRACKED, ViewValue::Bool(true)),
        ]
    }

    /// Paints an untracked bar at `elapsed` and reports `(spans, animating)`.
    fn untracked(elapsed: f32) -> (Vec<(f32, f32)>, bool) {
        let props = vec![
            ViewProperty::new(Progress::PROP_SHAPE, ViewValue::Int(0)),
            ViewProperty::new(Progress::PROP_TRACKED, ViewValue::Bool(false)),
        ];
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, Rect::new(Point::new(0.0, 0.0), BOX));
        let animating = paint_progress(&mut region, &CanvasPaint::new(&props, elapsed), accent());
        let _ = region.close();
        let spans = builder
            .build()
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, .. } => {
                    path.bounds().map(|b| (b.origin.x, b.right()))
                }
                _ => None,
            })
            .collect();
        (spans, animating)
    }

    fn accent() -> Brush {
        Brush::solid(Color::rgba(0.2, 0.5, 1.0, 1.0))
    }

    /// The strokes a paint produced, as `(command count, widths)`.
    fn strokes(value: f64, shape: ProgressShape) -> Vec<(f32, LineCap)> {
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, Rect::new(Point::new(0.0, 0.0), BOX));
        paint_progress(
            &mut region,
            &CanvasPaint::new(&props(value, shape), 0.0),
            accent(),
        );
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
    fn an_untracked_bar_moves_and_keeps_asking_for_frames() {
        // The two halves of an unbounded bar, and neither works without the
        // other: it must move, and it must be asked again or it moves once.
        let (early, animating) = untracked(0.1);
        assert!(animating, "an untracked bar votes for the next frame");
        let (later, _) = untracked(0.7);
        assert_ne!(
            early.last(),
            later.last(),
            "and the window is somewhere else a moment later: {early:?} vs {later:?}"
        );
    }

    #[test]
    fn a_tracked_bar_does_not_ask_for_frames() {
        // The other half: a bar that knows its value is redrawn when the value
        // changes, like anything else. Voting yes here would make an idle
        // application render forever.
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, Rect::new(Point::new(0.0, 0.0), BOX));
        let animating = paint_progress(
            &mut region,
            &CanvasPaint::new(&props(0.5, ProgressShape::Line), 3.0),
            accent(),
        );
        let _ = region.close();
        assert!(!animating);
    }

    #[test]
    fn the_sweep_window_never_appears_at_both_ends_at_once() {
        // Clamped rather than wrapped: a window that wrapped would read as two
        // bars rather than one travelling.
        for step in 0..40 {
            let (from, to) = super::sweep(step as f32 * 0.1);
            assert!(from <= to, "the window never inverts: {from}..{to}");
            assert!((0.0..=1.0).contains(&from) && (0.0..=1.0).contains(&to));
        }
    }

    #[test]
    fn a_node_carrying_no_properties_paints_an_empty_line() {
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, Rect::new(Point::new(0.0, 0.0), BOX));
        let _ = paint_progress(&mut region, &CanvasPaint::new(&[], 0.0), accent());
        assert!(region.close(), "and leaves the region balanced");
    }
}
