#![doc = include_str!("../../docs/api/slider/slider_painter.doc/_module.md")]

use obscur_geometry::Point;
use obscur_scene::color::Color;
use obscur_scene::command::CanvasRegion;
use obscur_scene::paint::{Brush, LineCap, Stroke};
use obscur_scene::path::Path;
use obscur_view::view::{
    CanvasPaint, SLIDER_HIGH, SLIDER_LOW, SLIDER_RANGED, SliderScale, ViewProperty, ViewValue,
};

/// How thick the track is, in logical pixels.
const TRACK_THICKNESS: f32 = 4.0;

#[doc = include_str!("../../docs/api/slider/slider_painter.doc/THUMB_RADIUS.md")]
const THUMB_RADIUS: f32 = 8.0;

/// Draws the track, the filled span and the thumbs.
pub fn paint_slider(region: &mut CanvasRegion<'_>, paint: &CanvasPaint<'_>, accent: Brush) -> bool {
    let props = paint.props();
    let size = region.size();
    if size.width <= 0.0 || size.height <= 0.0 {
        return false;
    }
    let scale = SliderScale::read(props);
    let ranged = read_flag(props, SLIDER_RANGED);
    let low = scale.fraction_of(number(props, SLIDER_LOW).unwrap_or(scale.min));
    let high = if ranged {
        scale
            .fraction_of(number(props, SLIDER_HIGH).unwrap_or(scale.min))
            .max(low)
    } else {
        low
    };

    // The track runs the full width, inset by the thumb's radius at each end, so
    // a thumb at either extreme sits inside the box rather than half outside it.
    let inset = THUMB_RADIUS.min(size.width / 2.0);
    let y = size.height / 2.0;
    let (left, right) = (inset, size.width - inset);
    let at = |fraction: f32| left + (right - left) * fraction;

    region.painter().stroke(
        line(left, right, y),
        Stroke::new(track_brush(), TRACK_THICKNESS).with_line_cap(LineCap::Round),
    );

    // A single slider fills from the start; a ranged one fills between its ends.
    // Reading both from the same pair is what keeps the two spellings one
    // drawing rather than two.
    let (from, to) = if ranged { (low, high) } else { (0.0, low) };
    if to > from {
        region.painter().stroke(
            line(at(from), at(to), y),
            Stroke::new(accent.clone(), TRACK_THICKNESS).with_line_cap(LineCap::Round),
        );
    }

    let radius = THUMB_RADIUS.min(size.height / 2.0);
    region
        .painter()
        .fill(thumb(at(low), y, radius), accent.clone());
    if ranged {
        region.painter().fill(thumb(at(high), y, radius), accent);
    }

    // Nothing moves on its own: a slider is redrawn when its value changes, like
    // any other control.
    false
}

/// A horizontal segment.
fn line(from_x: f32, to_x: f32, y: f32) -> Path {
    let mut path = Path::new();
    path.move_to(Point::new(from_x, y));
    path.line_to(Point::new(to_x, y));
    path
}

/// A circle, as four cubics.
fn thumb(cx: f32, cy: f32, radius: f32) -> Path {
    const K: f32 = 0.552_285;
    let mut path = Path::new();
    if radius <= 0.0 {
        return path;
    }
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
    path.close();
    path
}

/// The unfilled track's brush.
fn track_brush() -> Brush {
    Brush::solid(Color::rgba(0.17, 0.16, 0.15, 0.25))
}

/// A boolean property, defaulting to false.
fn read_flag(props: &[ViewProperty], name: obscur_identity::InternedSymbol) -> bool {
    props
        .iter()
        .find(|p| p.name() == name)
        .and_then(|p| match p.value() {
            ViewValue::Bool(v) => Some(*v),
            _ => None,
        })
        .unwrap_or(false)
}

/// A numeric property, whichever way it was written.
fn number(props: &[ViewProperty], name: obscur_identity::InternedSymbol) -> Option<f32> {
    props
        .iter()
        .find(|p| p.name() == name)
        .and_then(|p| match p.value() {
            #[allow(clippy::cast_possible_truncation)]
            ViewValue::Float(v) => Some(*v as f32),
            #[allow(clippy::cast_precision_loss)]
            ViewValue::Int(v) => Some(*v as f32),
            _ => None,
        })
        .filter(|v| v.is_finite())
}
