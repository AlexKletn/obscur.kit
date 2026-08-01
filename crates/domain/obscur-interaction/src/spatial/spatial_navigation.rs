#![doc = include_str!("spatial_navigation.doc/_module.md")]

use obscur_geometry::Rect;
use obscur_identity::NodeId;

use super::NavigationDirection;

#[doc = include_str!("spatial_navigation.doc/CROSS_AXIS_PENALTY.md")]
const CROSS_AXIS_PENALTY: f32 = 2.0;

#[doc = include_str!("spatial_navigation.doc/SpatialNavigation.md")]
#[derive(Debug, Default, Clone)]
pub struct SpatialNavigation;

impl SpatialNavigation {
    #[doc = include_str!("spatial_navigation.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[doc = include_str!("spatial_navigation.doc/find_next.md")]
    #[must_use]
    pub fn find_next(
        &self,
        from: Rect,
        candidates: &[(NodeId, Rect)],
        direction: NavigationDirection,
    ) -> Option<NodeId> {
        let (fx, fy) = center(from);

        let mut best: Option<(NodeId, f32)> = None;
        for &(node, rect) in candidates {
            let (cx, cy) = center(rect);
            let dx = cx - fx;
            let dy = cy - fy;

            // Reject candidates that are not strictly beyond `from` along the
            // direction's axis and sign, and derive the along/across distances.
            let (along, across) = match direction {
                NavigationDirection::Right if dx > 0.0 => (dx, dy.abs()),
                NavigationDirection::Left if dx < 0.0 => (-dx, dy.abs()),
                NavigationDirection::Down if dy > 0.0 => (dy, dx.abs()),
                NavigationDirection::Up if dy < 0.0 => (-dy, dx.abs()),
                _ => continue,
            };

            let score = along + CROSS_AXIS_PENALTY * across;
            let replace = match best {
                None => true,
                Some((best_node, best_score)) => {
                    score < best_score || (score == best_score && node < best_node)
                }
            };
            if replace {
                best = Some((node, score));
            }
        }

        best.map(|(node, _)| node)
    }
}

#[doc = include_str!("spatial_navigation.doc/center.md")]
fn center(rect: Rect) -> (f32, f32) {
    (
        rect.origin.x + rect.size.width / 2.0,
        rect.origin.y + rect.size.height / 2.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(w, h))
    }

    #[test]
    fn find_next_right_picks_nearest_aligned() {
        let nav = SpatialNavigation::new();
        let from = rect(0.0, 0.0, 10.0, 10.0);
        let candidates = [
            (node(1), rect(20.0, 0.0, 10.0, 10.0)),  // aligned, near
            (node(2), rect(40.0, 0.0, 10.0, 10.0)),  // aligned, far
            (node(3), rect(20.0, 50.0, 10.0, 10.0)), // near but misaligned
        ];
        assert_eq!(
            nav.find_next(from, &candidates, NavigationDirection::Right),
            Some(node(1))
        );
    }

    #[test]
    fn find_next_down_picks_nearest_aligned() {
        let nav = SpatialNavigation::new();
        let from = rect(0.0, 0.0, 10.0, 10.0);
        let candidates = [
            (node(1), rect(0.0, 20.0, 10.0, 10.0)),
            (node(2), rect(0.0, 60.0, 10.0, 10.0)),
        ];
        assert_eq!(
            nav.find_next(from, &candidates, NavigationDirection::Down),
            Some(node(1))
        );
    }

    #[test]
    fn find_next_ignores_candidates_behind() {
        let nav = SpatialNavigation::new();
        let from = rect(50.0, 50.0, 10.0, 10.0);
        // All candidates are to the left / above → none for Right.
        let candidates = [
            (node(1), rect(0.0, 50.0, 10.0, 10.0)),
            (node(2), rect(50.0, 0.0, 10.0, 10.0)),
        ];
        assert_eq!(
            nav.find_next(from, &candidates, NavigationDirection::Right),
            None
        );
    }

    #[test]
    fn find_next_tie_break_by_node_id() {
        let nav = SpatialNavigation::new();
        let from = rect(0.0, 0.0, 10.0, 10.0);
        // Two identical candidates (same score) → smaller NodeId wins.
        let candidates = [
            (node(9), rect(20.0, 0.0, 10.0, 10.0)),
            (node(4), rect(20.0, 0.0, 10.0, 10.0)),
        ];
        assert_eq!(
            nav.find_next(from, &candidates, NavigationDirection::Right),
            Some(node(4))
        );
    }

    #[test]
    fn find_next_empty_is_none() {
        let nav = SpatialNavigation::new();
        let from = rect(0.0, 0.0, 10.0, 10.0);
        assert_eq!(nav.find_next(from, &[], NavigationDirection::Right), None);
    }

    #[test]
    fn find_next_prefers_alignment_over_raw_distance() {
        let nav = SpatialNavigation::new();
        let from = rect(0.0, 0.0, 10.0, 10.0);
        // node(1): slightly farther along but perfectly aligned.
        // node(2): closer along but far off-axis → penalised.
        let candidates = [
            (node(1), rect(30.0, 0.0, 10.0, 10.0)),
            (node(2), rect(20.0, 40.0, 10.0, 10.0)),
        ];
        assert_eq!(
            nav.find_next(from, &candidates, NavigationDirection::Right),
            Some(node(1))
        );
    }
}
