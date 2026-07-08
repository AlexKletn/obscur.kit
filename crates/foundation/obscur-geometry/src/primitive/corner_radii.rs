//! Per-corner border radii in logical pixels.

use super::Rect;

/// Border radii for each corner, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRadii {
    /// Top-left radius.
    pub top_left: f32,
    /// Top-right radius.
    pub top_right: f32,
    /// Bottom-right radius.
    pub bottom_right: f32,
    /// Bottom-left radius.
    pub bottom_left: f32,
}

impl CornerRadii {
    /// The same radius on every corner.
    #[must_use]
    pub const fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Whether any corner has a positive radius (i.e. the box actually rounds).
    #[must_use]
    pub fn any_positive(self) -> bool {
        self.top_left > 0.0
            || self.top_right > 0.0
            || self.bottom_right > 0.0
            || self.bottom_left > 0.0
    }

    /// Offsets every corner by `delta`, keeping a sharp (zero) corner sharp.
    ///
    /// Per CSS Backgrounds §7.1.1 (`box-shadow` spread), the shadow shape's radii
    /// are the box radii offset by ±spread: an outset shadow grows them by
    /// `+spread`, an inset shadow shrinks them by `−spread`. A corner that is
    /// **already sharp** (radius `0`) stays sharp regardless of the offset; a
    /// rounded corner is offset and floored at `0` so it never goes negative.
    #[must_use]
    pub fn offset(self, delta: f32) -> Self {
        let f = |r: f32| if r > 0.0 { (r + delta).max(0.0) } else { 0.0 };
        Self {
            top_left: f(self.top_left),
            top_right: f(self.top_right),
            bottom_right: f(self.bottom_right),
            bottom_left: f(self.bottom_left),
        }
    }

    /// Applies the CSS Backgrounds §5.5 overlapping-curves correction for `rect`.
    ///
    /// When the radii on any side sum to more than that side's length the corners
    /// would overlap. The spec computes ONE uniform scale factor
    ///
    /// ```text
    /// f = min( W / (tl + tr),   // top side
    ///          W / (bl + br),   // bottom side
    ///          H / (tl + bl),   // left side
    ///          H / (tr + br) )  // right side
    /// ```
    ///
    /// over the four sides (a side whose radius-sum is `0` is skipped — treated as
    /// `+∞` so it never constrains, avoiding a divide-by-zero). If `f < 1` every
    /// corner radius is multiplied by `f`, preserving the proportions between
    /// corners; if `f ≥ 1` the radii already fit and are returned unchanged
    /// (byte-identical). This is the single scale applied to all four corners,
    /// unlike an independent per-corner `min(w,h)/2` clamp which distorts
    /// asymmetric radii on non-square boxes.
    ///
    /// `CornerRadii` is scalar-per-corner; per-axis elliptical corners (the full
    /// §5.5 `(x, y)` form) are a separate future item.
    #[must_use]
    pub fn normalized(self, rect: Rect) -> Self {
        let w = rect.size.width.max(0.0);
        let h = rect.size.height.max(0.0);
        // A zero-sum side does not constrain: treat its ratio as +∞.
        let ratio = |extent: f32, sum: f32| {
            if sum > 0.0 {
                extent / sum
            } else {
                f32::INFINITY
            }
        };
        let f = ratio(w, self.top_left + self.top_right)
            .min(ratio(w, self.bottom_left + self.bottom_right))
            .min(ratio(h, self.top_left + self.bottom_left))
            .min(ratio(h, self.top_right + self.bottom_right));
        if f < 1.0 {
            Self {
                top_left: self.top_left * f,
                top_right: self.top_right * f,
                bottom_right: self.bottom_right * f,
                bottom_left: self.bottom_left * f,
            }
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{Point, Size};

    fn rect(w: f32, h: f32) -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(w, h))
    }

    #[test]
    fn offset_grows_and_keeps_sharp_sharp() {
        let r = CornerRadii {
            top_left: 10.0,
            top_right: 0.0,
            bottom_right: 10.0,
            bottom_left: 10.0,
        };
        let out = r.offset(8.0);
        assert_eq!(out.top_left, 18.0);
        assert_eq!(out.top_right, 0.0, "a sharp corner stays sharp under +spread");
        assert_eq!(out.bottom_right, 18.0);
    }

    #[test]
    fn offset_shrinks_and_floors_at_zero() {
        let r = CornerRadii::uniform(10.0);
        assert_eq!(r.offset(-8.0), CornerRadii::uniform(2.0));
        // Over-shrink floors at 0, never negative.
        assert_eq!(r.offset(-40.0), CornerRadii::uniform(0.0));
        // Sharp corner stays 0 under −spread too.
        let sharp = CornerRadii::uniform(0.0);
        assert_eq!(sharp.offset(-8.0), CornerRadii::uniform(0.0));
    }

    #[test]
    fn normalized_scales_all_corners_uniformly_on_wide_short_box() {
        // 200×40 box, 30px on every corner. The top side sum (30+30=60) exceeds
        // neither 200, but the LEFT side sum (30+30=60) exceeds the height 40, so
        // f = 40/60 = 0.6666… governs — applied to ALL four corners.
        let r = CornerRadii::uniform(30.0);
        let out = r.normalized(rect(200.0, 40.0));
        let f = 40.0 / 60.0;
        let expected = 30.0 * f;
        for got in [out.top_left, out.top_right, out.bottom_right, out.bottom_left] {
            assert!((got - expected).abs() < 1e-4, "got {got}, want {expected}");
        }
        // None of the scaled radii exceeds the half-extents.
        assert!(out.top_left <= 20.0 + 1e-4);
    }

    #[test]
    fn normalized_leaves_fitting_radii_unchanged() {
        let r = CornerRadii::uniform(10.0);
        // 200×40 box: top sums 20 ≤ 200, side sums 20 ≤ 40 → f ≥ 1 → unchanged.
        assert_eq!(r.normalized(rect(200.0, 40.0)), r);
        let zero = CornerRadii::uniform(0.0);
        assert_eq!(zero.normalized(rect(200.0, 40.0)), zero);
    }

    #[test]
    fn normalized_wide_top_sum_overflow_scales_down() {
        // A wide-short box where tl+tr > width but each < min(w,h)/2 under the OLD
        // per-corner clamp: 120×40, tl=tr=70. Each 70 > min(120,40)/2 = 20 so the
        // old clamp would cap each at 20; the spec scales by f = min(120/140,
        // 40/140) = 40/140.
        let r = CornerRadii::uniform(70.0);
        let out = r.normalized(rect(120.0, 40.0));
        let f = 40.0 / 140.0;
        assert!((out.top_left - 70.0 * f).abs() < 1e-4);
        // Proportions preserved: all equal since input was uniform.
        assert_eq!(out.top_left, out.bottom_right);
    }
}
