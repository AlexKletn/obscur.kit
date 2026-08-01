#![doc = include_str!("gradient_brush.doc/_module.md")]

use obscur_geometry::{Point, Rect};
use obscur_scene::paint::{
    ConicGradient as SceneConicGradient, ExtendMode, GradientStop as SceneGradientStop,
    LinearGradient as SceneLinearGradient, RadialGradient as SceneRadialGradient,
};
use obscur_style::property::{
    ConicGradient, Corner, Length, LinearGradient, RadialGradient, RadialShape, RadialSize,
};

use super::paint_color::scene_color_from;

#[doc = include_str!("gradient_brush.doc/linear_gradient_brush.md")]
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

#[doc = include_str!("gradient_brush.doc/corner_direction.md")]
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

#[doc = include_str!("gradient_brush.doc/radial_gradient_brush.md")]
pub(super) fn radial_gradient_brush(
    gradient: &RadialGradient,
    bounds: Rect,
) -> SceneRadialGradient {
    let cx = resolve_center(gradient.center_x(), bounds.origin.x, bounds.size.width);
    let cy = resolve_center(gradient.center_y(), bounds.origin.y, bounds.size.height);
    let (rx, ry) = resolve_radii(gradient.shape(), gradient.size(), cx, cy, bounds);
    SceneRadialGradient::new_elliptical(
        Point::new(cx, cy),
        rx,
        ry,
        gradient_stops(gradient.stops()),
    )
}

#[doc = include_str!("gradient_brush.doc/conic_gradient_brush.md")]
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

#[doc = include_str!("gradient_brush.doc/gradient_stops.md")]
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

#[doc = include_str!("gradient_brush.doc/resolve_center.md")]
fn resolve_center(length: Length, origin: f32, extent: f32) -> f32 {
    match length {
        Length::Px(px) => origin + px.0,
        Length::Percent(pct) => origin + pct / 100.0 * extent,
        _ => origin + extent / 2.0,
    }
}

#[doc = include_str!("gradient_brush.doc/resolve_radii.md")]
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
                RadialSize::Length(_) => dx_far.hypot(dy_far),
            };
            (r, r)
        }
        RadialShape::Ellipse => match size {
            RadialSize::ClosestSide => (dx_near, dy_near),
            RadialSize::FarthestSide => (dx_far, dy_far),
            RadialSize::ClosestCorner => (dx_near * SQRT2, dy_near * SQRT2),
            RadialSize::FarthestCorner => (dx_far * SQRT2, dy_far * SQRT2),
            RadialSize::Length(len) => (resolve_axis_radius(len, w), resolve_axis_radius(len, h)),
        },
    }
}

#[doc = include_str!("gradient_brush.doc/resolve_axis_radius.md")]
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
