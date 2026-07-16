//! Per-corner border radii in logical pixels.

use super::Rect;

/// One corner's two radii, in logical pixels.
///
/// A CSS corner is a **quarter ellipse**, not a quarter circle: CSS Backgrounds 3
/// §4.1 defines the two `<length-percentage>` values of `border-*-radius` as "the
/// radii of a quarter ellipse that defines the shape of the corner", and resolves
/// them against different bases — "Percentages for the horizontal radius refer to
/// the width of the border box, whereas percentages for the vertical radius refer
/// to the height of the border box". So `border-radius: 50%` on a 200×100 box is
/// an ellipse (`x = 100`, `y = 50`), and only a square box makes it a circle. A
/// single scalar per corner cannot express that, which is why this type is a pair.
///
/// `x` runs along the horizontal edges, `y` along the vertical ones.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRadius {
    /// The horizontal radius (along the top/bottom edges).
    pub x: f32,
    /// The vertical radius (along the left/right edges).
    pub y: f32,
}

impl CornerRadius {
    /// A circular corner: the same radius on both axes.
    #[must_use]
    pub const fn circular(radius: f32) -> Self {
        Self {
            x: radius,
            y: radius,
        }
    }

    /// An elliptical corner with a horizontal radius `x` and a vertical radius `y`.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Whether this corner actually rounds.
    ///
    /// Per CSS Backgrounds 3 §4.1, "if either length is zero, the corner is
    /// square, not rounded" — a quarter ellipse with a zero axis has no area. So
    /// this is a conjunction, not a disjunction: `50%` on a zero-height box gives
    /// `y == 0` and a square corner.
    #[must_use]
    pub fn is_rounded(self) -> bool {
        self.x > 0.0 && self.y > 0.0
    }

    /// Both radii, floored at zero.
    ///
    /// A negative radius is not valid CSS and does not survive parsing; this is the
    /// defensive floor for a hand-built value reaching a path builder.
    #[must_use]
    pub fn non_negative(self) -> Self {
        Self {
            x: self.x.max(0.0),
            y: self.y.max(0.0),
        }
    }

    /// Scales both radii by `f`.
    #[must_use]
    pub fn scaled(self, f: f32) -> Self {
        Self {
            x: self.x * f,
            y: self.y * f,
        }
    }

    /// The radius of the circle standing in for this quarter ellipse, for a backend
    /// that cannot draw one — the mean of the two axes.
    ///
    /// **This is a lossy fallback, not a CSS rule.** No single radius describes a
    /// quarter ellipse, so any backend forced to pick one is approximating. The mean
    /// is chosen because it is *exact* whenever the corner is already circular
    /// (`x == y`, the case such a backend can express, so the common path never
    /// degrades) and unbiased otherwise — unlike `max(x, y)`, which systematically
    /// over-rounds, or `min`, which under-rounds.
    ///
    /// Every caller must be a backend that genuinely cannot express an ellipse, and
    /// must say so at the call site. Do **not** reach for this from scene geometry:
    /// [`super::super::Rect`]-based path building expresses ellipses exactly, and
    /// clip/scene geometry must agree across backends (spec §5.2).
    #[must_use]
    pub fn circular_approximation(self) -> f32 {
        (self.x + self.y) / 2.0
    }
}

/// Border radii for each corner, in logical pixels.
///
/// Each corner carries a horizontal and a vertical radius (see [`CornerRadius`]);
/// the common circular case sets both to the same number, which is what
/// [`CornerRadii::uniform`] builds.
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

    /// Whether any corner actually rounds (i.e. the box is not a plain rectangle).
    ///
    /// A corner counts only when **both** its radii are positive — see
    /// [`CornerRadius::is_rounded`].
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

    /// Offsets every corner by `delta` on **each axis independently**, keeping a
    /// sharp (zero) corner sharp and flooring at `0`.
    ///
    /// This is the neutral geometric primitive shared by the two CSS rules that
    /// grow or shrink a set of radii by a distance. It is deliberately *not* named
    /// for either of them, because they do not agree:
    ///
    /// * **CSS Backgrounds 3 §4.2 (corner shaping)** — "The padding edge (inner
    ///   border) radius is the outer border radius minus the corresponding border
    ///   thickness. In the case where this results in a negative value, the inner
    ///   radius is zero." That is exactly this method, and the *whole* of it: §4.2
    ///   prescribes a plain subtraction with no easing.
    /// * **CSS Backgrounds 3 §6.1.1 (`box-shadow` spread)** — the shadow's radii are
    ///   the box radii ±spread, "applied independently to the radii in each
    ///   dimension", which is this method **plus** a correction this method does not
    ///   implement: when a radius is *smaller* than the spread, §6.1.1 first scales
    ///   the spread by `1 + (r−1)³` (where `r` = radius ÷ spread) so a nearly-sharp
    ///   corner stays sharp rather than ballooning. A 10px radius with 20px spread
    ///   should give 27.5px; this method gives 30px.
    ///
    /// **Do not add the §6.1.1 easing here** — it would silently corrupt §4.2, which
    /// forbids it. Give spread its own method if it is ever closed.
    ///
    /// Sharp-stays-sharp is not a shortcut: it is the `r == 0` endpoint both rules
    /// agree on (§6.1.1's easing factor is `1 + (0−1)³ = 0` there), and it follows
    /// from §4.1 — a corner with a zero axis "is square, not rounded", so there is no
    /// curve for a distance to grow.
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

    /// Applies the CSS Backgrounds 3 §4.5 overlapping-curves correction for `rect`.
    ///
    /// The spec: "Let f = min(Li/Si), where i ∈ {top, right, bottom, left}, Si is
    /// the sum of the two corresponding radii of the corners on side i, and Ltop =
    /// Lbottom = the width of the box, and Lleft = Lright = the height of the box.
    /// If f < 1, then all corner radii are reduced by multiplying them by f."
    ///
    /// The *corresponding* radius of a corner on a side is the one running **along**
    /// that side: the horizontal (`x`) radii sum on the top and bottom, the vertical
    /// (`y`) radii on the left and right.
    ///
    /// ```text
    /// f = min( W / (tl.x + tr.x),   // top side
    ///          W / (bl.x + br.x),   // bottom side
    ///          H / (tl.y + bl.y),   // left side
    ///          H / (tr.y + br.y) )  // right side
    /// ```
    ///
    /// A side whose radius-sum is `0` is skipped — treated as `+∞` so it never
    /// constrains, avoiding a divide-by-zero. If `f < 1` **both** radii of every
    /// corner are multiplied by `f`, which is what the spec's note means by
    /// "quarter circles remain quarter circles and large radii remain larger than
    /// smaller ones"; if `f ≥ 1` the radii already fit and are returned unchanged
    /// (byte-identical). This is the single scale applied to all four corners,
    /// unlike an independent per-corner `min(w,h)/2` clamp which distorts
    /// asymmetric radii on non-square boxes.
    ///
    /// `border-radius: 50%` is the fixed point this protects: on any box the four
    /// side sums equal exactly that side's length, so `f == 1` and the authored
    /// ellipse survives untouched.
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
