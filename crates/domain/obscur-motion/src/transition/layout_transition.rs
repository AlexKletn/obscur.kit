#![doc = include_str!("layout_transition.doc/_module.md")]

use obscur_geometry::Rect;

use crate::track::Easing;
use crate::transition::TransitionDirection;
use crate::value::Interpolatable;

#[doc = include_str!("layout_transition.doc/LayoutTransition.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutTransition {
    from_bounds: Rect,
    to_bounds: Rect,
    easing: Easing,
    direction: TransitionDirection,
}

impl LayoutTransition {
    #[doc = include_str!("layout_transition.doc/new.md")]
    #[must_use]
    pub const fn new(from_bounds: Rect, to_bounds: Rect) -> Self {
        Self {
            from_bounds,
            to_bounds,
            easing: Easing::Linear,
            direction: TransitionDirection::Forward,
        }
    }

    #[doc = include_str!("layout_transition.doc/with_easing.md")]
    #[must_use]
    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    #[doc = include_str!("layout_transition.doc/with_direction.md")]
    #[must_use]
    pub const fn with_direction(mut self, direction: TransitionDirection) -> Self {
        self.direction = direction;
        self
    }

    #[doc = include_str!("layout_transition.doc/from_bounds.md")]
    #[must_use]
    pub const fn from_bounds(&self) -> Rect {
        self.from_bounds
    }

    #[doc = include_str!("layout_transition.doc/to_bounds.md")]
    #[must_use]
    pub const fn to_bounds(&self) -> Rect {
        self.to_bounds
    }

    #[doc = include_str!("layout_transition.doc/easing.md")]
    #[must_use]
    pub const fn easing(&self) -> Easing {
        self.easing
    }

    #[doc = include_str!("layout_transition.doc/direction.md")]
    #[must_use]
    pub const fn direction(&self) -> TransitionDirection {
        self.direction
    }

    #[doc = include_str!("layout_transition.doc/is_noop.md")]
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.from_bounds == self.to_bounds
    }

    #[doc = include_str!("layout_transition.doc/sample.md")]
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
