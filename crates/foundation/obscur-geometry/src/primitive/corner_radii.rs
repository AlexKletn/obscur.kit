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
    /// forbids it, and the two other callers that legitimately want a raw distance:
    /// the dashed-border centerline, and `outline`'s concentric growth (`outline` is
    /// CSS UI 3 §4 — it is not a CSS Backgrounds property at all — and neither it
    /// nor `outline-offset` (§4.5) defines any easing of its own). §6.1.1 has its own
    /// method: [`CornerRadii::spread_offset`].
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

    /// Offsets every corner by `delta` for CSS Backgrounds 3 **§6.1.1**
    /// (`box-shadow` spread), easing a corner whose radius is smaller than the
    /// distance it grows by.
    ///
    /// `delta` is the signed distance the *radii* grow by, which is **not** the
    /// authored `spread` for both shadow kinds: §6.1.1 expands an outer shadow's
    /// perimeter and contracts an inner one's ("expanded outward (for outer
    /// box-shadows) or contracted inward (for inner box-shadows)"), so a drop
    /// shadow passes `+spread` and an inset shadow passes `−spread`. Growth is
    /// therefore `delta > 0` whichever kind is asking.
    ///
    /// Against [`CornerRadii::offset`] — the neutral §4.2 primitive — this adds
    /// exactly one rule, and nothing else. §6.1.1:
    ///
    /// > However, in order to create a sharper corner when the border radius is
    /// > small (and thus ensure continuity between round and sharp corners), when
    /// > the border radius is less than the spread distance (or in the case of an
    /// > inner shadow, less than the absolute value of a negative spread
    /// > distance), the spread distance is first multiplied by the proportion
    /// > 1 + (r-1)³, where r is the ratio of the border radius to the spread
    /// > distance, in calculating the corner radii of the spread shadow shape. For
    /// > example, if the border radius is 10px and the spread distance is 20px
    /// > (r = .5), the corner radius of the shadow shape will be
    /// > 10px + 20px × (1 + (.5 - 1)³) = 27.5px rather than 30px. This adjustment
    /// > is applied independently to the radii in each dimension.
    ///
    /// **Only growth is eased.** Both cases the spec names — an outer shadow whose
    /// radius is under a *positive* spread, and an inner shadow whose radius is
    /// under the absolute value of a *negative* one — are cases where the radius
    /// grows; §8.3's change log records that narrowing as its own entry ("Spread
    /// radius adjustment is only applied to shadows and margins where the radius of
    /// curvature grows, rather than shrinks", Issue 1900). A shrinking corner
    /// (`delta < 0`) therefore takes the plain floored subtraction — the same
    /// arithmetic [`CornerRadii::offset`] does.
    ///
    /// "Applied independently to the radii in each dimension" is why `r` is taken
    /// per axis rather than per corner: an elliptical corner can sit below the
    /// spread on one axis and above it on the other, and each axis eases alone.
    ///
    /// The two branches meet continuously: at `radius == delta`, `r == 1` and the
    /// factor is `1 + 0³ == 1`, which is the raw addition. At the other endpoint a
    /// sharp corner stays sharp — `r == 0` gives `1 + (−1)³ == 0` — which is also
    /// what the shared `is_rounded` gate enforces. Those two agreeing endpoints are
    /// why a test that only probes them cannot tell this method from `offset`.
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

/// One axis of CSS Backgrounds 3 §6.1.1's spread rule: `radius` grown by `delta`,
/// with the easing applied only when the corner grows by more than it is round.
///
/// See [`CornerRadii::spread_offset`] for the quoted text and why growth is the
/// only eased direction.
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
