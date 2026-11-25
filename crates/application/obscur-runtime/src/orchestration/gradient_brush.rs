//! CSS Images 3 gradients resolved into scene fill brushes: the linear
//! gradient line, the radial ending shape, and the conic sweep, each over the
//! box they paint.

use obscur_geometry::{Point, Rect};
use obscur_scene::paint::{
    ConicGradient as SceneConicGradient, ExtendMode, GradientStop as SceneGradientStop,
    LinearGradient as SceneLinearGradient, RadialGradient as SceneRadialGradient,
};
use obscur_style::property::{
    ConicGradient, Corner, Length, LinearGradient, RadialGradient, RadialShape, RadialSize,
};

use super::paint_color::scene_color_from;

#[doc = include_str!("../../docs/api/orchestration/gradient_brush.doc/linear_gradient_brush.md")]
pub(super) fn linear_gradient_brush(
    gradient: &LinearGradient,
    bounds: Rect,
) -> SceneLinearGradient {
    let w = bounds.size.width;
    let h = bounds.size.height;
    let cx = bounds.origin.x + w / 2.0;
    let cy = bounds.origin.y + h / 2.0;
    // Unit direction of increasing gradient offset (0% → 100%), in screen space
    // (y down). A `to <corner>` uses the aspect-ratio-aware direction (spec §3.1);
    // otherwise the angle's (sin θ, −cos θ).
    let (dir_x, dir_y) = gradient.corner().map_or_else(
        || {
            let theta = gradient.angle_deg().to_radians();
            (theta.sin(), -theta.cos())
        },
        |corner| corner_direction(corner, w, h),
    );
    // Gradient-line length = |w·dir_x| + |h·dir_y| (spec §3.1 projection note),
    // valid for any unit `dir` (reduces to |w·sin θ| + |h·cos θ| for an angle).
    let half_len = (w * dir_x).abs().mul_add(0.5, (h * dir_y).abs() * 0.5);
    let start = Point::new(cx - dir_x * half_len, cy - dir_y * half_len);
    if let Some(positions) = gradient.repeat_positions() {
        let (first, period, stops) =
            resolve_repeat_stops(gradient.stops(), positions, f64::from(2.0 * half_len));
        if period <= 0.0 || !period.is_finite() {
            return SceneLinearGradient::new(
                start,
                Point::new(cx + dir_x * half_len, cy + dir_y * half_len),
                vec![SceneGradientStop::new(0.0, average_repeat_stops(&stops))],
            );
        }
        let point_at = |distance: f64| {
            Point::new(
                obscur_geometry::saturating_f32(f64::from(start.x) + f64::from(dir_x) * distance),
                obscur_geometry::saturating_f32(f64::from(start.y) + f64::from(dir_y) * distance),
            )
        };
        let start = point_at(first);
        let end = point_at(first + period);
        return SceneLinearGradient::new(start, end, phase_repeat_stops(&stops, 0.0))
            .with_extend(ExtendMode::Repeat);
    }
    let stops = gradient_stops(gradient.stops());
    // A repeating gradient's axis spans exactly one period from the box's 0%
    // anchor, and the pattern tiles with `ExtendMode::Repeat`.
    if let Some(period) = gradient.period() {
        let line_len = 2.0 * half_len;
        let period_px = match period {
            Length::Px(px) => px.0,
            Length::Percent(pct) => pct / 100.0 * line_len,
            _ => line_len,
        };
        let end = Point::new(start.x + dir_x * period_px, start.y + dir_y * period_px);
        return SceneLinearGradient::new(start, end, stops).with_extend(ExtendMode::Repeat);
    }
    let end = Point::new(cx + dir_x * half_len, cy + dir_y * half_len);
    SceneLinearGradient::new(start, end, stops)
}

#[doc = include_str!("../../docs/api/orchestration/gradient_brush.doc/corner_direction.md")]
fn corner_direction(corner: Corner, w: f32, h: f32) -> (f32, f32) {
    let (sx, sy) = match corner {
        Corner::TopRight => (h, -w),
        Corner::BottomRight => (h, w),
        Corner::BottomLeft => (-h, w),
        Corner::TopLeft => (-h, -w),
    };
    let len = sx.hypot(sy);
    if len == 0.0 {
        (0.0, 1.0)
    } else {
        (sx / len, sy / len)
    }
}

#[doc = include_str!("../../docs/api/orchestration/gradient_brush.doc/radial_gradient_brush.md")]
pub(super) fn radial_gradient_brush(
    gradient: &RadialGradient,
    bounds: Rect,
) -> SceneRadialGradient {
    let cx = resolve_center(gradient.center_x(), bounds.origin.x, bounds.size.width);
    let cy = resolve_center(gradient.center_y(), bounds.origin.y, bounds.size.height);
    let (rx, ry) = resolve_radii(gradient.shape(), gradient.size(), cx, cy, bounds);
    let center = Point::new(cx, cy);
    if let Some(positions) = gradient.repeat_positions() {
        return repeating_radial_brush(gradient, positions, center, rx, ry);
    }
    SceneRadialGradient::new_elliptical(center, rx, ry, gradient_stops(gradient.stops()))
}

/// Resolves authored stop lengths and maps one CSS radial period to the repeating scene ellipse.
fn repeating_radial_brush(
    gradient: &RadialGradient,
    positions: &[Option<Length>],
    center: Point,
    rx: f32,
    ry: f32,
) -> SceneRadialGradient {
    let radius = f64::from(rx);
    let (first, period, stops) = resolve_repeat_stops(gradient.stops(), positions, radius);
    let period_x = obscur_geometry::saturating_f32(period);
    let period_y = obscur_geometry::saturating_f32(period * f64::from(ry) / radius);
    if !period.is_finite()
        || period_x <= 0.0
        || period_y <= 0.0
        || !period_y.is_finite()
        || radius <= 0.0
    {
        return SceneRadialGradient::new(
            center,
            1.0,
            vec![SceneGradientStop::new(0.0, average_repeat_stops(&stops))],
        );
    }
    let phase = (-first / period).rem_euclid(1.0);
    let phase = if phase == 1.0 { 0.0 } else { phase };
    let stops = phase_repeat_stops(&stops, phase);
    SceneRadialGradient::new_elliptical(center, period_x, period_y, stops)
        .with_extend(ExtendMode::Repeat)
}

/// Resolves CSS stop distances and normalizes one repeat, retaining the first stop’s offset.
fn resolve_repeat_stops(
    stops: &[obscur_style::property::GradientStop],
    positions: &[Option<Length>],
    basis: f64,
) -> (f64, f64, Vec<(f64, obscur_scene::color::Color)>) {
    let mut offsets: Vec<_> = positions
        .iter()
        .map(|position| match position {
            Some(Length::Px(px)) => Some(f64::from(px.0)),
            Some(Length::Percent(percent)) => Some(basis * f64::from(*percent) / 100.0),
            _ => None,
        })
        .collect();
    let last = offsets.len() - 1;
    offsets[0].get_or_insert(0.0);
    offsets[last].get_or_insert(basis);
    let mut previous = f64::NEG_INFINITY;
    for offset in offsets.iter_mut().flatten() {
        *offset = offset.max(previous);
        previous = *offset;
    }
    let first = offsets[0].unwrap_or_default();
    let period = previous - first;
    let colors = stops.iter().map(|stop| scene_color_from(stop.color()));
    let mut start = 0;
    for end in 1..=last {
        let Some(next) = offsets[end] else {
            continue;
        };
        let from = offsets[start].unwrap_or_default();
        for (index, offset) in offsets.iter_mut().enumerate().take(end).skip(start + 1) {
            #[allow(clippy::cast_precision_loss)]
            let t = (index - start) as f64 / (end - start) as f64;
            *offset = Some(from + (next - from) * t);
        }
        start = end;
    }
    let real_period = period > 0.0 && period.is_finite();
    let stops: Vec<_> = offsets
        .into_iter()
        .zip(colors)
        .enumerate()
        .map(|(index, (offset, color))| {
            #[allow(clippy::cast_precision_loss)]
            let normalized = if real_period {
                (offset.unwrap_or_default() - first) / period
            } else {
                index as f64 / last as f64
            };
            (normalized, color)
        })
        .collect();
    (first, period, stops)
}

/// Computes the CSS Images average of a collapsed repeat in premultiplied sRGBA.
#[allow(clippy::cast_possible_truncation)]
fn average_repeat_stops(stops: &[(f64, obscur_scene::color::Color)]) -> obscur_scene::color::Color {
    use obscur_scene::color::Color;

    let mut total = [0.0; 4];
    for pair in stops.windows(2) {
        let weight = (pair[1].0 - pair[0].0) / 2.0;
        for (_, color) in pair {
            let alpha = f64::from(color.a);
            total[0] += f64::from(color.r) * alpha * weight;
            total[1] += f64::from(color.g) * alpha * weight;
            total[2] += f64::from(color.b) * alpha * weight;
            total[3] += alpha * weight;
        }
    }
    let [r, g, b, a] = total;
    if a > 0.0 {
        Color::rgba((r / a) as f32, (g / a) as f32, (b / a) as f32, a as f32)
    } else {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    }
}

/// Converts a normalized repeat to scene stops and shifts its seam when a nonzero phase is needed.
#[allow(clippy::cast_possible_truncation)]
fn phase_repeat_stops(
    stops: &[(f64, obscur_scene::color::Color)],
    phase: f64,
) -> Vec<SceneGradientStop> {
    use obscur_scene::color::convert::{oklab_lerp, premul_oklab};

    if phase == 0.0 {
        return stops
            .iter()
            .map(|&(offset, color)| SceneGradientStop::new(offset as f32, color))
            .collect();
    }
    let lower = stops.partition_point(|&(offset, _)| offset < phase);
    let upper = stops.partition_point(|&(offset, _)| offset <= phase);
    let (before, after) = if upper > lower {
        (stops[lower].1, stops[upper - 1].1)
    } else {
        let (a, from) = stops[lower - 1];
        let (b, to) = stops[upper];
        let color = oklab_lerp(premul_oklab(from), premul_oklab(to), (phase - a) / (b - a));
        (color, color)
    };
    let mut rotated = Vec::with_capacity(stops.len() + 2);
    rotated.push(SceneGradientStop::new(0.0, after));
    rotated.extend(
        stops[upper..]
            .iter()
            .map(|&(offset, color)| SceneGradientStop::new((offset - phase) as f32, color)),
    );
    rotated.extend(
        stops[..lower]
            .iter()
            .map(|&(offset, color)| SceneGradientStop::new((1.0 + offset - phase) as f32, color)),
    );
    rotated.push(SceneGradientStop::new(1.0, before));
    rotated
}

#[doc = include_str!("../../docs/api/orchestration/gradient_brush.doc/conic_gradient_brush.md")]
pub(super) fn conic_gradient_brush(gradient: &ConicGradient, bounds: Rect) -> SceneConicGradient {
    let cx = resolve_center(gradient.center_x(), bounds.origin.x, bounds.size.width);
    let cy = resolve_center(gradient.center_y(), bounds.origin.y, bounds.size.height);
    let start_angle = (gradient.from_angle_deg() - 90.0).to_radians();
    SceneConicGradient::new(
        Point::new(cx, cy),
        start_angle,
        gradient_stops(gradient.stops()),
    )
}

/// Maps a style gradient's color stops to scene [`SceneGradientStop`]s: each stop's
/// resolved `0.0..=1.0` position becomes the scene offset and its
/// [`StyleColor`](obscur_style::property::StyleColor)
/// the scene paint color. An unpositioned stop (never produced by the CSS lowering,
/// which resolves every position) falls back to its index-proportional offset.
fn gradient_stops(stops: &[obscur_style::property::GradientStop]) -> Vec<SceneGradientStop> {
    stops
        .iter()
        .enumerate()
        .map(|(index, stop)| {
            let offset = stop.position().unwrap_or_else(|| {
                let last = stops.len().saturating_sub(1).max(1);
                #[allow(clippy::cast_precision_loss)]
                {
                    index as f32 / last as f32
                }
            });
            SceneGradientStop::new(offset, scene_color_from(stop.color()))
        })
        .collect()
}

/// Resolves a gradient's box-relative center [`Length`] on one axis to an absolute
/// coordinate: an absolute `px` offset from `origin`, a `Percent` as a fraction of
/// the axis `extent` from `origin`. Any other (context-dependent) length falls back
/// to the axis center (a documented approximation).
fn resolve_center(length: Length, origin: f32, extent: f32) -> f32 {
    match length {
        Length::Px(px) => origin + px.0,
        Length::Percent(pct) => origin + pct / 100.0 * extent,
        _ => origin + extent / 2.0,
    }
}

#[doc = include_str!("../../docs/api/orchestration/gradient_brush.doc/resolve_radii.md")]
fn resolve_radii(
    shape: RadialShape,
    size: RadialSize,
    cx: f32,
    cy: f32,
    bounds: Rect,
) -> (f32, f32) {
    const SQRT2: f32 = std::f32::consts::SQRT_2;
    let (left, top, right, bottom) = (
        bounds.origin.x,
        bounds.origin.y,
        bounds.right(),
        bounds.bottom(),
    );
    let dx_near = (cx - left).abs().min((right - cx).abs());
    let dx_far = (cx - left).abs().max((right - cx).abs());
    let dy_near = (cy - top).abs().min((bottom - cy).abs());
    let dy_far = (cy - top).abs().max((bottom - cy).abs());
    let (w, h) = (bounds.size.width, bounds.size.height);
    match shape {
        RadialShape::Circle => {
            let r = match size {
                RadialSize::ClosestSide => dx_near.min(dy_near),
                RadialSize::FarthestSide => dx_far.max(dy_far),
                RadialSize::ClosestCorner => dx_near.hypot(dy_near),
                RadialSize::FarthestCorner => dx_far.hypot(dy_far),
                RadialSize::Length(Length::Px(px)) => px.0,
                RadialSize::Length(Length::Percent(pct)) => pct / 100.0 * w,
                RadialSize::Length(_) | RadialSize::Ellipse(..) => dx_far.hypot(dy_far),
            };
            (r, r)
        }
        RadialShape::Ellipse => match size {
            RadialSize::ClosestSide => (dx_near, dy_near),
            RadialSize::FarthestSide => (dx_far, dy_far),
            RadialSize::ClosestCorner => (dx_near * SQRT2, dy_near * SQRT2),
            RadialSize::FarthestCorner => (dx_far * SQRT2, dy_far * SQRT2),
            RadialSize::Length(len) => (resolve_axis_radius(len, w), resolve_axis_radius(len, h)),
            RadialSize::Ellipse(x, y) => (resolve_axis_radius(x, w), resolve_axis_radius(y, h)),
        },
    }
}

/// Resolves an explicit radius [`Length`] against a box axis of length `extent`:
/// an absolute `px` directly, a percentage as that fraction of `extent`; any other
/// (context-dependent) length falls back to the full `extent`.
fn resolve_axis_radius(len: Length, extent: f32) -> f32 {
    match len {
        Length::Px(px) => px.0,
        Length::Percent(pct) => pct / 100.0 * extent,
        _ => extent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};
    use obscur_style::property::StyleColor;

    #[test]
    fn linear_corner_direction_is_aspect_ratio_aware() {
        use obscur_style::property::GradientStop as StyleStop;

        // `linear-gradient(to top right, black, white)` on a WIDE 200×50 box.
        let stops = vec![
            StyleStop::new(StyleColor::BLACK, 0.0),
            StyleStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
        ];
        let gradient = LinearGradient::new_corner(Corner::TopRight, 45.0, stops);
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 50.0));
        let g = linear_gradient_brush(&gradient, bounds);

        // Direction points toward the actual top-right corner: normalize(h, −w) =
        // normalize(50, −200), NOT a 45° diagonal (spec §3.1).
        let dir = Point::new(g.end().x - g.start().x, g.end().y - g.start().y);
        let len = dir.x.hypot(dir.y);
        let (ux, uy) = (dir.x / len, dir.y / len);
        let expected = 50.0_f32.hypot(200.0);
        assert!((ux - 50.0 / expected).abs() < 1e-3, "dir.x {ux}");
        assert!((uy - (-200.0 / expected)).abs() < 1e-3, "dir.y {uy}");
        // Emphatically not 45° (sin45 ≈ 0.707).
        assert!(
            (ux - std::f32::consts::FRAC_1_SQRT_2).abs() > 0.4,
            "not 45°"
        );
        // Endpoints are symmetric about the box center (100, 25).
        let midx = (g.start().x + g.end().x) / 2.0;
        let midy = (g.start().y + g.end().y) / 2.0;
        assert!((midx - 100.0).abs() < 1e-3 && (midy - 25.0).abs() < 1e-3);
    }

    #[test]
    fn linear_angle_gradient_line_length_matches_formula() {
        use obscur_style::property::GradientStop as StyleStop;

        // `linear-gradient(45deg, …)` line length = |W·sin45| + |H·cos45|.
        let stops = vec![
            StyleStop::new(StyleColor::BLACK, 0.0),
            StyleStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
        ];
        let gradient = LinearGradient::new(45.0, stops);
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 50.0));
        let g = linear_gradient_brush(&gradient, bounds);
        let line_len = (g.end().x - g.start().x).hypot(g.end().y - g.start().y);
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let expected = (200.0 * s).abs() + (50.0 * s).abs();
        assert!(
            (line_len - expected).abs() < 1e-3,
            "line {line_len} vs {expected}"
        );
    }

    #[test]
    fn resolve_radii_circle_extents() {
        // Centered in a 100×50 box: center (50, 25).
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 50.0));
        // closest-side circle = min distance to a side = 25 (half height).
        let (rx, ry) = resolve_radii(
            RadialShape::Circle,
            RadialSize::ClosestSide,
            50.0,
            25.0,
            bounds,
        );
        assert!(
            (rx - 25.0).abs() < 1e-3 && (ry - 25.0).abs() < 1e-3,
            "closest-side {rx},{ry}"
        );
        // farthest-corner circle = hypot(50, 25).
        let (rx, ry) = resolve_radii(
            RadialShape::Circle,
            RadialSize::FarthestCorner,
            50.0,
            25.0,
            bounds,
        );
        let want = 50.0_f32.hypot(25.0);
        assert!(
            (rx - want).abs() < 1e-3 && (rx - ry).abs() < 1e-6,
            "farthest-corner {rx},{ry}"
        );
    }

    #[test]
    fn resolve_radii_ellipse_nonsquare_has_rx_ne_ry() {
        // Centered in a WIDE 480×120 box: center (240, 60).
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(480.0, 120.0));
        // farthest-corner ellipse keeps the far-side ratio, scaled by √2:
        // (dx_far, dy_far)·√2 = (240√2, 60√2).
        let (rx, ry) = resolve_radii(
            RadialShape::Ellipse,
            RadialSize::FarthestCorner,
            240.0,
            60.0,
            bounds,
        );
        let s = std::f32::consts::SQRT_2;
        assert!((rx - 240.0 * s).abs() < 1e-2, "rx {rx}");
        assert!((ry - 60.0 * s).abs() < 1e-2, "ry {ry}");
        assert!(
            rx > ry,
            "a non-square ellipse is stretched: rx {rx} > ry {ry}"
        );
        // closest-side ellipse = the per-axis near distances (240, 60), still rx≠ry.
        let (rx, ry) = resolve_radii(
            RadialShape::Ellipse,
            RadialSize::ClosestSide,
            240.0,
            60.0,
            bounds,
        );
        assert!(
            (rx - 240.0).abs() < 1e-3 && (ry - 60.0).abs() < 1e-3,
            "closest-side {rx},{ry}"
        );
    }
}
