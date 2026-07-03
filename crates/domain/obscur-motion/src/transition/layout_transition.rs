//! The [`LayoutTransition`] FLIP-style layout-box animation (spec §4.17
//! "layout-affecting animations", "before/after layout geometry").

use obscur_geometry::Rect;

use crate::track::Easing;
use crate::transition::TransitionDirection;
use crate::value::Interpolatable;

/// A FLIP-style animation of an element's layout box (spec §4.17
/// "layout-affecting animations").
///
/// *FLIP* stands for **F**irst (the box *before* a layout change), **L**ast (the
/// box *after*), **I**nvert and **P**lay: given the before/after bounds, the
/// element is animated from the old box to the new one so a layout jump reads as
/// smooth movement. This type is the domain half — it holds the two bounds, an
/// easing curve and a play [`direction`](TransitionDirection), and interpolates
/// between them. Capturing the actual before/after geometry is the runtime's
/// job, performed in its pre/post-layout phases, which then feed those rects into
/// [`from_bounds`](Self::from_bounds) and [`to_bounds`](Self::to_bounds).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutTransition {
    from_bounds: Rect,
    to_bounds: Rect,
    easing: Easing,
    direction: TransitionDirection,
}

impl LayoutTransition {
    /// Creates a layout transition between `from_bounds` (before) and
    /// `to_bounds` (after), playing [`Linear`](Easing::Linear) and
    /// [`Forward`](TransitionDirection::Forward) (spec §4.17).
    #[must_use]
    pub const fn new(from_bounds: Rect, to_bounds: Rect) -> Self {
        Self {
            from_bounds,
            to_bounds,
            easing: Easing::Linear,
            direction: TransitionDirection::Forward,
        }
    }

    /// Returns this transition with `easing` applied (spec §4.17).
    #[must_use]
    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Returns this transition with `direction` applied (spec §4.17).
    #[must_use]
    pub const fn with_direction(mut self, direction: TransitionDirection) -> Self {
        self.direction = direction;
        self
    }

    /// The element's layout box *before* the change (spec §4.17).
    #[must_use]
    pub const fn from_bounds(&self) -> Rect {
        self.from_bounds
    }

    /// The element's layout box *after* the change (spec §4.17).
    #[must_use]
    pub const fn to_bounds(&self) -> Rect {
        self.to_bounds
    }

    /// The easing curve applied to progress (spec §4.17).
    #[must_use]
    pub const fn easing(&self) -> Easing {
        self.easing
    }

    /// The direction this transition plays in (spec §4.17).
    #[must_use]
    pub const fn direction(&self) -> TransitionDirection {
        self.direction
    }

    /// Returns `true` if the box does not move (`from_bounds == to_bounds`), so
    /// the transition produces no visible motion (spec §4.17).
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.from_bounds == self.to_bounds
    }

    /// Samples the interpolated layout box at raw progress `t` (spec §4.17).
    ///
    /// `t` is oriented by the [`direction`](Self::direction) (clamped to
    /// `[0, 1]`), shaped by the [`easing`](Self::easing), then used to
    /// interpolate from [`from_bounds`](Self::from_bounds) toward
    /// [`to_bounds`](Self::to_bounds). Pure and deterministic.
    #[must_use]
    pub fn sample(&self, t: f32) -> Rect {
        let oriented = self.direction.orient(t);
        let eased = self.easing.apply(oriented);
        self.from_bounds.interpolate(&self.to_bounds, eased)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(w, h))
    }

    #[test]
    fn new_defaults_to_linear_forward() {
        let t = LayoutTransition::new(rect(0.0, 0.0, 10.0, 10.0), rect(4.0, 4.0, 20.0, 20.0));
        assert_eq!(t.easing(), Easing::Linear);
        assert_eq!(t.direction(), TransitionDirection::Forward);
    }

    #[test]
    fn sample_endpoints_are_from_and_to() {
        let from = rect(0.0, 0.0, 10.0, 10.0);
        let to = rect(4.0, 8.0, 20.0, 40.0);
        let t = LayoutTransition::new(from, to);
        assert_eq!(t.sample(0.0), from);
        assert_eq!(t.sample(1.0), to);
    }

    #[test]
    fn sample_midpoint_interpolates() {
        let from = rect(0.0, 0.0, 0.0, 0.0);
        let to = rect(4.0, 8.0, 20.0, 40.0);
        let t = LayoutTransition::new(from, to);
        assert_eq!(t.sample(0.5), rect(2.0, 4.0, 10.0, 20.0));
    }

    #[test]
    fn reverse_direction_swaps_endpoints() {
        let from = rect(0.0, 0.0, 10.0, 10.0);
        let to = rect(4.0, 8.0, 20.0, 40.0);
        let t = LayoutTransition::new(from, to).with_direction(TransitionDirection::Reverse);
        // Raw t=0 lands on the target, raw t=1 lands on the origin.
        assert_eq!(t.sample(0.0), to);
        assert_eq!(t.sample(1.0), from);
    }

    #[test]
    fn easing_shapes_the_sampled_box() {
        let from = rect(0.0, 0.0, 0.0, 0.0);
        let to = rect(0.0, 0.0, 0.0, 100.0);
        let linear = LayoutTransition::new(from, to);
        let ease_in = linear.with_easing(Easing::EaseIn);
        // EaseIn(0.5) == 0.25, so it lags the linear midpoint of 50.
        assert_eq!(linear.sample(0.5).size.height, 50.0);
        assert_eq!(ease_in.sample(0.5).size.height, 25.0);
    }

    #[test]
    fn is_noop_when_bounds_are_equal() {
        let b = rect(1.0, 2.0, 3.0, 4.0);
        assert!(LayoutTransition::new(b, b).is_noop());
        assert!(!LayoutTransition::new(b, rect(1.0, 2.0, 3.0, 5.0)).is_noop());
    }
}
