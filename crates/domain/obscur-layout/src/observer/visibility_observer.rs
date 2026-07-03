//! The [`VisibilityObserver`] — watches whether a node is visible (spec §4.4).
//!
//! Like all geometry observers it reports the coalesced net change after layout
//! commit. A node is *visible* when its box has non-zero area and — if a
//! viewport is supplied — its box overlaps that viewport. The observer fires
//! only on a visible/hidden transition.

use obscur_geometry::Rect;
use obscur_identity::NodeId;

use crate::model::LayoutBox;

/// A visibility transition reported by a [`VisibilityObserver`] (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibilityChange {
    /// The observed node.
    pub node: NodeId,
    /// The node's visibility after the transition.
    pub visible: bool,
}

impl VisibilityChange {
    /// Constructs a visibility change event.
    #[must_use]
    pub const fn new(node: NodeId, visible: bool) -> Self {
        Self { node, visible }
    }
}

/// Watches a node's visibility — non-zero area and (optionally) overlap with a
/// viewport — firing on each visible/hidden transition (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibilityObserver {
    node: NodeId,
    last: Option<bool>,
}

impl VisibilityObserver {
    /// Creates an observer for `node`. It has not yet observed visibility, so
    /// the first [`observe`](Self::observe) fires with the initial value.
    #[must_use]
    pub const fn new(node: NodeId) -> Self {
        Self { node, last: None }
    }

    /// The observed node.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    /// The last observed visibility, or `None` before the first observation.
    #[must_use]
    pub const fn last(self) -> Option<bool> {
        self.last
    }

    /// Records the newly-committed box and optional `viewport`, returning a
    /// [`VisibilityChange`] on a visible/hidden transition (or on the first
    /// observation).
    ///
    /// The node is visible when its box area is greater than zero and, when a
    /// `viewport` is given, the box overlaps it. Returns `None` when visibility
    /// is unchanged.
    #[must_use]
    pub fn observe(&mut self, box_: LayoutBox, viewport: Option<Rect>) -> Option<VisibilityChange> {
        let size = box_.size();
        let has_area = size.width > 0.0 && size.height > 0.0;
        let visible = has_area && viewport.is_none_or(|v| rects_overlap(v, box_.bounds));
        if self.last == Some(visible) {
            return None;
        }
        self.last = Some(visible);
        Some(VisibilityChange::new(self.node, visible))
    }

    /// Forgets the last observed visibility, so the next observation fires
    /// afresh.
    pub const fn reset(&mut self) {
        self.last = None;
    }
}

/// Returns whether two rectangles share any interior area (half-open edges: a
/// shared edge alone does not overlap).
fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.origin.x < b.right()
        && b.origin.x < a.right()
        && a.origin.y < b.bottom()
        && b.origin.y < a.bottom()
}

#[cfg(test)]
mod tests {
    use super::{VisibilityChange, VisibilityObserver};
    use crate::model::LayoutBox;
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::NodeId;

    fn box_at(x: f32, y: f32, w: f32, h: f32) -> LayoutBox {
        LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h)))
    }

    #[test]
    fn first_observe_fires_with_initial_value() {
        let node = NodeId::from_raw(1);
        let mut obs = VisibilityObserver::new(node);
        let b = box_at(0.0, 0.0, 10.0, 10.0);
        assert_eq!(obs.observe(b, None), Some(VisibilityChange::new(node, true)));
    }

    #[test]
    fn zero_area_box_is_hidden() {
        let node = NodeId::from_raw(2);
        let mut obs = VisibilityObserver::new(node);
        let b = box_at(0.0, 0.0, 0.0, 10.0);
        assert_eq!(obs.observe(b, None), Some(VisibilityChange::new(node, false)));
    }

    #[test]
    fn visible_to_hidden_transition_with_viewport() {
        let node = NodeId::from_raw(4);
        let mut obs = VisibilityObserver::new(node);
        let viewport = Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0));

        // Inside the viewport → visible (first observe fires).
        let inside = box_at(10.0, 10.0, 20.0, 20.0);
        assert_eq!(
            obs.observe(inside, Some(viewport)),
            Some(VisibilityChange::new(node, true))
        );

        // Same box again → no transition.
        assert_eq!(obs.observe(inside, Some(viewport)), None);

        // Moved fully outside → visible→hidden transition.
        let outside = box_at(200.0, 200.0, 20.0, 20.0);
        assert_eq!(
            obs.observe(outside, Some(viewport)),
            Some(VisibilityChange::new(node, false))
        );
    }

    #[test]
    fn reset_makes_next_observe_fire() {
        let mut obs = VisibilityObserver::new(NodeId::from_raw(1));
        let b = box_at(0.0, 0.0, 5.0, 5.0);
        let _ = obs.observe(b, None);
        obs.reset();
        assert!(obs.observe(b, None).is_some());
    }
}
