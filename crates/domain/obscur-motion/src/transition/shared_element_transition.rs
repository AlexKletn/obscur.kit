//! The [`SharedElementTransition`] cross-view element morph and its
//! [`SharedElementKey`] (spec §4.17 "shared-element transitions", "stable
//! identity").

use obscur_geometry::Rect;

use crate::track::Easing;
use crate::transition::TransitionDirection;
use crate::value::Interpolatable;

/// A stable tag identifying the *same* element across two views (spec §4.17
/// "stable identity").
///
/// When a shared-element transition runs, the runtime pairs elements by matching
/// keys: an element tagged `SharedElementKey("avatar")` in the old view morphs
/// into the element with the same key in the new view. This is a *local* newtype
/// — the motion crate owns and compares these tags itself and does not depend on
/// any shared identity crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SharedElementKey(Box<str>);

impl SharedElementKey {
    /// Creates a key from any string-like tag (spec §4.17).
    #[must_use]
    pub fn new(tag: impl Into<Box<str>>) -> Self {
        Self(tag.into())
    }

    /// Returns the tag as a string slice (spec §4.17).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Morphs a shared element from its box in an OLD view to its box in a NEW view
/// (spec §4.17 "shared-element transitions").
///
/// Identified by a stable [`SharedElementKey`], the element preserves its
/// identity across the view change while its layout box animates from
/// [`from_bounds`](Self::from_bounds) (its position/size in the old view) to
/// [`to_bounds`](Self::to_bounds) (its position/size in the new view) — the same
/// FLIP interpolation as [`LayoutTransition`](super::LayoutTransition). The
/// runtime pairs elements by key across views and supplies the from/to bounds;
/// this type holds them and interpolates.
#[derive(Debug, Clone, PartialEq)]
pub struct SharedElementTransition {
    key: SharedElementKey,
    from_bounds: Rect,
    to_bounds: Rect,
    easing: Easing,
    direction: TransitionDirection,
}

impl SharedElementTransition {
    /// Creates a shared-element transition for `key`, morphing from
    /// `from_bounds` (old view) to `to_bounds` (new view), playing
    /// [`Linear`](Easing::Linear) and
    /// [`Forward`](TransitionDirection::Forward) (spec §4.17).
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

    /// Returns this transition with `easing` applied (spec §4.17).
    #[must_use]
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Returns this transition with `direction` applied (spec §4.17).
    #[must_use]
    pub fn with_direction(mut self, direction: TransitionDirection) -> Self {
        self.direction = direction;
        self
    }

    /// The stable key pairing this element across the two views (spec §4.17).
    #[must_use]
    pub const fn key(&self) -> &SharedElementKey {
        &self.key
    }

    /// The element's box in the OLD view (spec §4.17).
    #[must_use]
    pub const fn from_bounds(&self) -> Rect {
        self.from_bounds
    }

    /// The element's box in the NEW view (spec §4.17).
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

    /// Returns `true` if `key` identifies this transition's element (spec
    /// §4.17 "stable identity").
    #[must_use]
    pub fn matches(&self, key: &SharedElementKey) -> bool {
        &self.key == key
    }

    /// Samples the interpolated element box at raw progress `t` (spec §4.17).
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
