#![doc = include_str!("shared_element_transition.doc/_module.md")]

use obscur_geometry::Rect;

use crate::track::Easing;
use crate::transition::TransitionDirection;
use crate::value::Interpolatable;

#[doc = include_str!("shared_element_transition.doc/SharedElementKey.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SharedElementKey(Box<str>);

impl SharedElementKey {
    #[doc = include_str!("shared_element_transition.doc/new.md")]
    #[must_use]
    pub fn new(tag: impl Into<Box<str>>) -> Self {
        Self(tag.into())
    }

    #[doc = include_str!("shared_element_transition.doc/as_str.md")]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[doc = include_str!("shared_element_transition.doc/SharedElementTransition.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SharedElementTransition {
    key: SharedElementKey,
    from_bounds: Rect,
    to_bounds: Rect,
    easing: Easing,
    direction: TransitionDirection,
}

impl SharedElementTransition {
    #[doc = include_str!("shared_element_transition.doc/new_2.md")]
    #[must_use]
    pub fn new(key: SharedElementKey, from_bounds: Rect, to_bounds: Rect) -> Self {
        Self {
            key,
            from_bounds,
            to_bounds,
            easing: Easing::Linear,
            direction: TransitionDirection::Forward,
        }
    }

    #[doc = include_str!("shared_element_transition.doc/with_easing.md")]
    #[must_use]
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    #[doc = include_str!("shared_element_transition.doc/with_direction.md")]
    #[must_use]
    pub fn with_direction(mut self, direction: TransitionDirection) -> Self {
        self.direction = direction;
        self
    }

    #[doc = include_str!("shared_element_transition.doc/key.md")]
    #[must_use]
    pub const fn key(&self) -> &SharedElementKey {
        &self.key
    }

    #[doc = include_str!("shared_element_transition.doc/from_bounds.md")]
    #[must_use]
    pub const fn from_bounds(&self) -> Rect {
        self.from_bounds
    }

    #[doc = include_str!("shared_element_transition.doc/to_bounds.md")]
    #[must_use]
    pub const fn to_bounds(&self) -> Rect {
        self.to_bounds
    }

    #[doc = include_str!("shared_element_transition.doc/easing.md")]
    #[must_use]
    pub const fn easing(&self) -> Easing {
        self.easing
    }

    #[doc = include_str!("shared_element_transition.doc/direction.md")]
    #[must_use]
    pub const fn direction(&self) -> TransitionDirection {
        self.direction
    }

    #[doc = include_str!("shared_element_transition.doc/matches.md")]
    #[must_use]
    pub fn matches(&self, key: &SharedElementKey) -> bool {
        &self.key == key
    }

    #[doc = include_str!("shared_element_transition.doc/sample.md")]
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
    fn key_round_trips_tag() {
        let key = SharedElementKey::new("avatar");
        assert_eq!(key.as_str(), "avatar");
    }

    #[test]
    fn matches_only_same_key() {
        let t = SharedElementTransition::new(
            SharedElementKey::new("avatar"),
            rect(0.0, 0.0, 10.0, 10.0),
            rect(4.0, 4.0, 20.0, 20.0),
        );
        assert_eq!(t.key(), &SharedElementKey::new("avatar"));
        assert!(t.matches(&SharedElementKey::new("avatar")));
        assert!(!t.matches(&SharedElementKey::new("header")));
    }

    #[test]
    fn sample_endpoints_are_from_and_to() {
        let from = rect(0.0, 0.0, 10.0, 10.0);
        let to = rect(4.0, 8.0, 20.0, 40.0);
        let t = SharedElementTransition::new(SharedElementKey::new("k"), from, to);
        assert_eq!(t.sample(0.0), from);
        assert_eq!(t.sample(1.0), to);
    }

    #[test]
    fn with_direction_reverse_swaps_endpoints() {
        let from = rect(0.0, 0.0, 10.0, 10.0);
        let to = rect(4.0, 8.0, 20.0, 40.0);
        let t = SharedElementTransition::new(SharedElementKey::new("k"), from, to)
            .with_direction(TransitionDirection::Reverse);
        assert_eq!(t.sample(0.0), to);
        assert_eq!(t.sample(1.0), from);
    }
}
