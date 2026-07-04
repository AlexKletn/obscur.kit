//! The [`IntersectionObserver`] — watches how much a node overlaps a root rect
//! (spec §4.4).
//!
//! Modelled on the web `IntersectionObserver`: it tracks the fraction of the
//! target rectangle covered by a root/viewport rectangle and fires when that
//! ratio crosses its `threshold`, i.e. when the target transitions between
//! *intersecting* and *not intersecting*. Like all geometry observers it
//! reports the coalesced net change after layout commit.

use obscur_geometry::Rect;
use obscur_identity::NodeId;

/// An intersection transition reported by an [`IntersectionObserver`]
/// (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntersectionChange {
    /// The observed node.
    pub node: NodeId,
    /// The fraction (`0.0..=1.0`) of the target covered by the root at the
    /// transition.
    pub ratio: f32,
    /// Whether the target is intersecting (`ratio >= threshold`) after the
    /// transition.
    pub intersecting: bool,
}

impl IntersectionChange {
    /// Constructs an intersection change event.
    #[must_use]
    pub const fn new(node: NodeId, ratio: f32, intersecting: bool) -> Self {
        Self {
            node,
            ratio,
            intersecting,
        }
    }
}

/// Watches the overlap ratio of a node's rectangle against a root rectangle,
/// firing when the target crosses the `threshold` (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntersectionObserver {
    node: NodeId,
    threshold: f32,
    last_intersecting: Option<bool>,
}

impl IntersectionObserver {
    /// Creates an observer for `node` with the given `threshold`, clamped to
    /// `0.0..=1.0`. It has not yet observed intersection, so the first
    /// [`observe`](Self::observe) fires with the initial value.
    #[must_use]
    pub fn new(node: NodeId, threshold: f32) -> Self {
        Self {
            node,
            threshold: threshold.clamp(0.0, 1.0),
            last_intersecting: None,
        }
    }

    /// The observed node.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    /// The (clamped) intersection threshold.
    #[must_use]
    pub const fn threshold(self) -> f32 {
        self.threshold
    }

    /// The last observed intersecting state, or `None` before the first
    /// observation.
    #[must_use]
    pub const fn last_intersecting(self) -> Option<bool> {
        self.last_intersecting
    }

    /// Records the current `target` and `root` rectangles, returning an
    /// [`IntersectionChange`] when the target crosses the threshold (or on the
    /// first observation).
    ///
    /// The ratio is the overlap area divided by the target area (`0.0` when the
    /// target has zero area). The target is intersecting when
    /// `ratio >= threshold`. Returns `None` when the intersecting state is
    /// unchanged.
    #[must_use]
    pub fn observe(&mut self, target: Rect, root: Rect) -> Option<IntersectionChange> {
        let target_area = target.size.width * target.size.height;
        let ratio = if target_area > 0.0 {
            overlap_area(target, root) / target_area
        } else {
            0.0
        };
        let intersecting = ratio >= self.threshold;
        if self.last_intersecting == Some(intersecting) {
            return None;
        }
        self.last_intersecting = Some(intersecting);
        Some(IntersectionChange::new(self.node, ratio, intersecting))
    }

    /// Forgets the last observed intersecting state, so the next observation
    /// fires afresh.
    pub const fn reset(&mut self) {
        self.last_intersecting = None;
    }
}

/// The area of the axis-aligned overlap of two rectangles (`0.0` when they do
/// not overlap).
fn overlap_area(a: Rect, b: Rect) -> f32 {
    let width = (a.right().min(b.right()) - a.origin.x.max(b.origin.x)).max(0.0);
    let height = (a.bottom().min(b.bottom()) - a.origin.y.max(b.origin.y)).max(0.0);
    width * height
}

#[cfg(test)]
mod tests {
    use super::{IntersectionObserver, overlap_area};
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::NodeId;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(w, h))
    }

    #[test]
    fn fully_inside_is_intersecting() {
        let node = NodeId::from_raw(1);
        let mut obs = IntersectionObserver::new(node, 0.5);
        let root = rect(0.0, 0.0, 100.0, 100.0);
        let target = rect(10.0, 10.0, 20.0, 20.0);
        let change = obs.observe(target, root).expect("first observe fires");
        assert!(change.intersecting);
        assert!((change.ratio - 1.0).abs() < 1e-6);
    }

    #[test]
    fn fully_outside_is_not_intersecting() {
        let node = NodeId::from_raw(2);
        let mut obs = IntersectionObserver::new(node, 0.5);
        let root = rect(0.0, 0.0, 100.0, 100.0);
        let target = rect(200.0, 200.0, 20.0, 20.0);
        let change = obs.observe(target, root).expect("first observe fires");
        assert!(!change.intersecting);
        assert!(change.ratio.abs() < 1e-6);
    }

    #[test]
    fn half_overlap_against_threshold_fires_only_on_transition() {
        let node = NodeId::from_raw(3);
        let mut obs = IntersectionObserver::new(node, 0.5);
        let root = rect(0.0, 0.0, 100.0, 100.0);

        // Half of the target sticks out to the right → ratio 0.5 >= 0.5.
        let half = rect(90.0, 10.0, 20.0, 20.0);
        let change = obs.observe(half, root).expect("first observe fires");
        assert!(change.intersecting);
        assert!((change.ratio - 0.5).abs() < 1e-6);

        // Same overlap → no transition.
        assert_eq!(obs.observe(half, root), None);

        // Only a quarter overlaps → ratio 0.25 < 0.5 → transition to false.
        let quarter = rect(95.0, 10.0, 20.0, 20.0);
        let change = obs.observe(quarter, root).expect("crosses below threshold");
        assert!(!change.intersecting);
        assert!((change.ratio - 0.25).abs() < 1e-6);
    }

    #[test]
    fn zero_area_target_has_zero_ratio() {
        let mut obs = IntersectionObserver::new(NodeId::from_raw(4), 0.5);
        let root = rect(0.0, 0.0, 100.0, 100.0);
        let target = rect(10.0, 10.0, 0.0, 20.0);
        let change = obs.observe(target, root).expect("first observe fires");
        assert!(!change.intersecting);
        assert!(change.ratio.abs() < 1e-6);
    }

    #[test]
    fn threshold_is_clamped() {
        assert!(
            (IntersectionObserver::new(NodeId::from_raw(1), 5.0).threshold() - 1.0).abs() < 1e-6
        );
        assert!(
            IntersectionObserver::new(NodeId::from_raw(1), -2.0)
                .threshold()
                .abs()
                < 1e-6
        );
    }

    #[test]
    fn overlap_area_of_disjoint_rects_is_zero() {
        let a = rect(0.0, 0.0, 10.0, 10.0);
        let b = rect(20.0, 20.0, 10.0, 10.0);
        assert!(overlap_area(a, b).abs() < 1e-6);
    }

    #[test]
    fn reset_makes_next_observe_fire() {
        let mut obs = IntersectionObserver::new(NodeId::from_raw(5), 0.5);
        let root = rect(0.0, 0.0, 100.0, 100.0);
        let target = rect(10.0, 10.0, 20.0, 20.0);
        let _ = obs.observe(target, root);
        obs.reset();
        assert!(obs.observe(target, root).is_some());
    }
}
