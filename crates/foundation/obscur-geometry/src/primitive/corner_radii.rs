//! Per-corner border radii in logical pixels.

use super::Rect;

use super::CornerRadius;

#[doc = include_str!("../../docs/api/primitive/corner_radii.doc/CornerRadii.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRadii {
    /// Top-left radii.
    pub top_left: CornerRadius,
    /// Top-right radii.
    pub top_right: CornerRadius,
    /// Bottom-right radii.
    pub bottom_right: CornerRadius,
    /// Bottom-left radii.
    pub bottom_left: CornerRadius,
}

impl CornerRadii {
    /// The same **circular** radius on every corner.
    #[must_use]
    pub const fn uniform(radius: f32) -> Self {
        Self::all(CornerRadius::circular(radius))
    }

    /// The same **elliptical** radii on every corner: `x` horizontal, `y` vertical.
    #[must_use]
    pub const fn elliptical(x: f32, y: f32) -> Self {
        Self::all(CornerRadius::new(x, y))
    }

    /// The same `corner` on every corner.
    #[must_use]
    pub const fn all(corner: CornerRadius) -> Self {
        Self {
            top_left: corner,
            top_right: corner,
            bottom_right: corner,
            bottom_left: corner,
        }
    }

    /// The four corners in CSS order: top-left, top-right, bottom-right, bottom-left.
    #[must_use]
    pub const fn corners(self) -> [CornerRadius; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }

    #[doc = include_str!("../../docs/api/primitive/corner_radii.doc/any_positive.md")]
    #[must_use]
    pub fn any_positive(self) -> bool {
        self.corners().iter().any(|c| c.is_rounded())
    }

    /// Every corner floored at zero on both axes.
    #[must_use]
    pub fn non_negative(self) -> Self {
        self.map(CornerRadius::non_negative)
    }

    /// Applies `f` to every corner.
    #[must_use]
    pub fn map(self, f: impl Fn(CornerRadius) -> CornerRadius) -> Self {
        Self {
            top_left: f(self.top_left),
            top_right: f(self.top_right),
            bottom_right: f(self.bottom_right),
            bottom_left: f(self.bottom_left),
        }
    }

    #[doc = include_str!("../../docs/api/primitive/corner_radii.doc/offset.md")]
    #[must_use]
    pub fn offset(self, delta: f32) -> Self {
        self.map(|c| {
            if c.is_rounded() {
                CornerRadius::new((c.x + delta).max(0.0), (c.y + delta).max(0.0))
            } else {
                CornerRadius::default()
            }
        })
    }

    #[doc = include_str!("../../docs/api/primitive/corner_radii.doc/spread_offset.md")]
    #[must_use]
    pub fn spread_offset(self, delta: f32) -> Self {
        self.map(|c| {
            if c.is_rounded() {
                CornerRadius::new(spread_axis(c.x, delta), spread_axis(c.y, delta))
            } else {
                CornerRadius::default()
            }
        })
    }

    #[doc = include_str!("../../docs/api/primitive/corner_radii.doc/normalized.md")]
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
        let f = ratio(w, self.top_left.x + self.top_right.x)
            .min(ratio(w, self.bottom_left.x + self.bottom_right.x))
            .min(ratio(h, self.top_left.y + self.bottom_left.y))
            .min(ratio(h, self.top_right.y + self.bottom_right.y));
        if f < 1.0 {
            self.map(|c| c.scaled(f))
        } else {
            self
        }
    }
}

#[doc = include_str!("../../docs/api/primitive/corner_radii.doc/spread_axis.md")]
fn spread_axis(radius: f32, delta: f32) -> f32 {
    // Shrinking (§8.3 Issue 1900), or a corner already at least as round as the
    // distance it grows by (`r ≥ 1`, outside the rule's "border radius is less
    // than the spread distance" trigger): the distance is added raw.
    if delta <= 0.0 || radius >= delta {
        return (radius + delta).max(0.0);
    }
    // r = the ratio of the border radius to the spread distance, in [0, 1) here.
    let r = radius / delta;
    radius + delta * (1.0 + (r - 1.0).powi(3))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{Point, Size};

    fn rect(w: f32, h: f32) -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(w, h))
    }

    #[test]
    fn uniform_is_circular_on_every_corner() {
        let r = CornerRadii::uniform(4.0);
        for c in r.corners() {
            assert_eq!(c, CornerRadius::new(4.0, 4.0));
        }
    }

    #[test]
    fn a_corner_with_one_zero_axis_is_square() {
        // Spec §4.1: "If either length is zero, the corner is square, not rounded."
        assert!(!CornerRadius::new(10.0, 0.0).is_rounded());
        assert!(!CornerRadius::new(0.0, 10.0).is_rounded());
        assert!(CornerRadius::new(10.0, 1.0).is_rounded());
        assert!(!CornerRadii::elliptical(10.0, 0.0).any_positive());
        assert!(CornerRadii::elliptical(10.0, 5.0).any_positive());
    }

    #[test]
    fn offset_grows_and_keeps_sharp_sharp() {
        let r = CornerRadii {
            top_left: CornerRadius::circular(10.0),
            top_right: CornerRadius::default(),
            bottom_right: CornerRadius::circular(10.0),
            bottom_left: CornerRadius::circular(10.0),
        };
        let out = r.offset(8.0);
        assert_eq!(out.top_left, CornerRadius::circular(18.0));
        assert_eq!(
            out.top_right,
            CornerRadius::default(),
            "a sharp corner stays sharp under +spread"
        );
        assert_eq!(out.bottom_right, CornerRadius::circular(18.0));
    }

    #[test]
    fn offset_moves_both_axes_of_an_elliptical_corner() {
        // Spread is a distance, not a scale: it adds to each axis independently, so
        // an ellipse's two radii stay `spread` apart from the box's, not in ratio.
        let out = CornerRadii::elliptical(40.0, 10.0).offset(5.0);
        assert_eq!(out.top_left, CornerRadius::new(45.0, 15.0));
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
    fn spread_offset_eases_a_radius_smaller_than_the_spread() {
        // §6.1.1's own worked example, verbatim: "if the border radius is 10px and
        // the spread distance is 20px (r = .5), the corner radius of the shadow
        // shape will be 10px + 20px × (1 + (.5 - 1)³) = 27.5px rather than 30px."
        // 30px is exactly what the raw `offset` primitive gives, so this number is
        // the whole difference between the two methods.
        let out = CornerRadii::uniform(10.0).spread_offset(20.0);
        assert!(
            (out.top_left.x - 27.5).abs() < 1e-4 && (out.top_left.y - 27.5).abs() < 1e-4,
            "§6.1.1 worked example: want 27.5 on both axes, got {:?}",
            out.top_left
        );
    }

    #[test]
    fn offset_keeps_section_4_2_a_plain_subtraction() {
        // The guard on the fix that would have been wrong. §4.2: "The padding edge
        // (inner border) radius is the outer border radius minus the corresponding
        // border thickness. In the case where this results in a negative value, the
        // inner radius is zero." A plain subtraction — no easing, at any ratio of
        // radius to thickness. If §6.1.1's easing ever migrates into `offset`, a
        // 10px outer radius with a 20px border stops flooring at 0 and this fails.
        assert_eq!(
            CornerRadii::uniform(10.0).offset(-20.0),
            CornerRadii::uniform(0.0),
            "§4.2's negative case floors at zero with no easing"
        );
        assert_eq!(
            CornerRadii::uniform(30.0).offset(-4.0),
            CornerRadii::uniform(26.0),
            "§4.2 is outer radius minus thickness, exactly"
        );
        // And the growth direction, which is where §6.1.1 diverges: `offset` must
        // still add the distance RAW — 30, not §6.1.1's 27.5. This is the literal
        // line the two rules disagree on.
        assert_eq!(
            CornerRadii::uniform(10.0).offset(20.0),
            CornerRadii::uniform(30.0),
            "`offset` is the neutral primitive: it must never ease"
        );
    }

    #[test]
    fn spread_offset_does_not_ease_a_radius_at_or_above_the_spread() {
        // The rule triggers only "when the border radius is less than the spread
        // distance". At r ≥ 1 the distance is added raw, so these agree with
        // `offset` exactly.
        assert_eq!(
            CornerRadii::uniform(20.0).spread_offset(20.0),
            CornerRadii::uniform(40.0),
            "r == 1 is the crossover: the factor is 1 + 0³ == 1, i.e. raw addition"
        );
        assert_eq!(
            CornerRadii::uniform(50.0).spread_offset(20.0),
            CornerRadii::uniform(70.0)
        );
    }

    #[test]
    fn spread_offset_never_eases_a_shrinking_corner() {
        // §8.3 (Issue 1900): "Spread radius adjustment is only applied to shadows
        // and margins where the radius of curvature grows, rather than shrinks."
        // A shrinking corner takes the plain floored subtraction. If the easing
        // were applied blind to sign, a 10px radius shrinking by 20px would take
        // the `r = -0.5` branch and land somewhere other than the floor.
        assert_eq!(
            CornerRadii::uniform(10.0).spread_offset(-20.0),
            CornerRadii::uniform(0.0)
        );
        assert_eq!(
            CornerRadii::uniform(10.0).spread_offset(-4.0),
            CornerRadii::uniform(6.0),
            "shrinking is a plain subtraction, matching `offset`"
        );
        // Every shrinking distance is the plain floored subtraction. Spelled out
        // literally rather than compared against `offset`, so this test cannot be
        // carried by §4.2's path — it must fail on its own if the easing leaks
        // into the shrink branch.
        for (d, want) in [
            (-1.0_f32, 9.0_f32),
            (-5.0, 5.0),
            (-9.5, 0.5),
            (-10.0, 0.0),
            (-100.0, 0.0),
        ] {
            assert_eq!(
                CornerRadii::uniform(10.0).spread_offset(d),
                CornerRadii::uniform(want),
                "spread_offset({d}) on a 10px radius must be a plain floored subtraction"
            );
        }
    }

    #[test]
    fn spread_offset_eases_an_inner_shadow_with_a_negative_spread() {
        // The spec's second trigger: "or in the case of an inner shadow, less than
        // the absolute value of a negative spread distance". An inner shadow
        // contracts by `spread`, so it passes `−spread`; a negative authored spread
        // (`box-shadow: inset ... -20px`) therefore GROWS the radii by 20 and must
        // ease exactly as the outer case does — same 27.5.
        let authored_spread = -20.0_f32;
        let out = CornerRadii::uniform(10.0).spread_offset(-authored_spread);
        assert!(
            (out.top_left.x - 27.5).abs() < 1e-4,
            "an inset shadow's negative spread grows the radius and must ease: got {:?}",
            out.top_left
        );
    }

    #[test]
    fn spread_offset_eases_each_axis_of_an_ellipse_independently() {
        // "This adjustment is applied independently to the radii in each
        // dimension." This corner is BELOW the spread on x (10 < 20 → eased to
        // 27.5) and ABOVE it on y (50 ≥ 20 → raw 70). A per-corner decision — one
        // ratio for both axes — would have to pick one branch and get the other
        // axis wrong.
        let out = CornerRadii::elliptical(10.0, 50.0).spread_offset(20.0);
        assert!(
            (out.top_left.x - 27.5).abs() < 1e-4,
            "x is under the spread and must ease, got {}",
            out.top_left.x
        );
        assert!(
            (out.top_left.y - 70.0).abs() < 1e-4,
            "y is over the spread and must not ease, got {}",
            out.top_left.y
        );
    }

    #[test]
    fn spread_offset_keeps_a_sharp_corner_sharp() {
        // The r == 0 endpoint, where the easing factor is 1 + (−1)³ == 0. Both
        // rules agree here, which is why this endpoint cannot discriminate the two
        // methods — it is asserted for the shape, not as evidence of the easing.
        let r = CornerRadii {
            top_left: CornerRadius::circular(10.0),
            top_right: CornerRadius::default(),
            bottom_right: CornerRadius::circular(10.0),
            bottom_left: CornerRadius::circular(10.0),
        };
        let out = r.spread_offset(20.0);
        assert_eq!(out.top_right, CornerRadius::default());
        assert!((out.top_left.x - 27.5).abs() < 1e-4);
    }

    #[test]
    fn spread_offset_is_monotonic_and_continuous_across_the_crossover() {
        // "ensure continuity between round and sharp corners" — the eased branch
        // must meet the raw branch at r == 1 with no step, and must never invert
        // (a rounder box must not yield a sharper shadow).
        let delta = 20.0_f32;
        let mut prev = f32::NEG_INFINITY;
        let mut radius = 0.0_f32;
        while radius <= 40.0 {
            let got = CornerRadii::uniform(radius.max(1e-3))
                .spread_offset(delta)
                .top_left
                .x;
            assert!(
                got >= prev - 1e-3,
                "spread_offset inverted at radius {radius}"
            );
            prev = got;
            radius += 0.25;
        }
        // No step at the crossover: approaching r == 1 from below lands on raw.
        let below = CornerRadii::uniform(19.99).spread_offset(delta).top_left.x;
        let at = CornerRadii::uniform(20.0).spread_offset(delta).top_left.x;
        assert!(
            (at - below).abs() < 0.05,
            "discontinuity at r == 1: {below} then {at}"
        );
        // And the eased result always sits BELOW the raw one in between — that is
        // the point of the rule ("create a sharper corner when the border radius
        // is small").
        for radius in [1.0_f32, 5.0, 10.0, 15.0, 19.0] {
            let eased = CornerRadii::uniform(radius).spread_offset(delta).top_left.x;
            // The raw sum spelled out, not taken from `offset`: this test must
            // stand on §6.1.1's arithmetic alone.
            let raw = radius + delta;
            assert!(
                eased < raw,
                "at radius {radius}: eased {eased} !< raw {raw}"
            );
        }
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
        for c in out.corners() {
            assert!(
                (c.x - expected).abs() < 1e-4 && (c.y - expected).abs() < 1e-4,
                "got {c:?}, want {expected} on both axes"
            );
        }
        // None of the scaled radii exceeds the half-extents.
        assert!(out.top_left.y <= 20.0 + 1e-4);
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
        assert!((out.top_left.x - 70.0 * f).abs() < 1e-4);
        // Proportions preserved: all equal since input was uniform.
        assert_eq!(out.top_left, out.bottom_right);
    }

    #[test]
    fn normalized_leaves_a_fifty_percent_ellipse_alone() {
        // The circle/ellipse idiom is §4.5's fixed point: `50%` on a 200×100 box is
        // (100, 50) per corner, every side sum equals that side's length exactly, so
        // f == 1 and the ellipse must survive. A per-axis-blind correction would
        // shrink it here and turn the pill back into a rounded rectangle.
        let r = CornerRadii::elliptical(100.0, 50.0);
        assert_eq!(r.normalized(rect(200.0, 100.0)), r);
    }

    #[test]
    fn normalized_measures_each_side_against_its_own_axis() {
        // A tall box with wide-but-short corners: the horizontal radii sum to more
        // than the WIDTH while the vertical radii fit the height easily. Only a
        // correction that sums `x` on the top side catches this.
        let r = CornerRadii::elliptical(80.0, 5.0);
        let out = r.normalized(rect(100.0, 400.0));
        // Top side: 80+80=160 > 100 → f = 100/160 = 0.625. Left side: 5+5=10 ≤ 400.
        let f = 0.625;
        assert!((out.top_left.x - 50.0).abs() < 1e-4);
        assert!(
            (out.top_left.y - 5.0 * f).abs() < 1e-4,
            "the spec scales ALL radii by f, including the axis that fit"
        );
    }
}
