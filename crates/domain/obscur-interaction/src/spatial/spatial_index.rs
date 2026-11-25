//! The [`SpatialIndex`] — the incremental, z-order-aware index backing hit
//! testing (spec §4.7).

use std::collections::{BTreeMap, btree_map};

use obscur_geometry::{Point, Rect};
use obscur_identity::NodeId;

use super::{HitTest, spatial_grid::SpatialGrid};

use super::Entry;

#[doc = include_str!("../../docs/api/spatial/spatial_index.doc/SpatialIndex.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpatialIndex {
    entries: BTreeMap<NodeId, Entry>,
    grid: SpatialGrid,
}

impl SpatialIndex {
    /// Creates an empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces the entry for `node` — the incremental update.
    pub fn insert(&mut self, node: NodeId, bounds: Rect, z_order: i32) {
        let entry = Entry::new(bounds, z_order);
        match self.entries.entry(node) {
            btree_map::Entry::Vacant(slot) => {
                self.grid.insert(node, bounds);
                slot.insert(entry);
            }
            btree_map::Entry::Occupied(mut slot) if *slot.get() != entry => {
                let old_bounds = slot.get().bounds;
                if old_bounds != bounds {
                    self.grid.remove(node, old_bounds);
                    self.grid.insert(node, bounds);
                }
                slot.insert(entry);
            }
            btree_map::Entry::Occupied(_) => {}
        }
    }

    /// Keeps the entries whose node IDs satisfy the predicate.
    pub fn retain(&mut self, mut keep: impl FnMut(NodeId) -> bool) {
        self.entries.retain(|&node, entry| {
            let retained = keep(node);
            if !retained {
                self.grid.remove(node, entry.bounds);
            }
            retained
        });
    }

    /// Removes `node` from the index, returning whether it was present.
    pub fn remove(&mut self, node: NodeId) -> bool {
        let Some(entry) = self.entries.remove(&node) else {
            return false;
        };
        self.grid.remove(node, entry.bounds);
        true
    }

    /// Updates only the `bounds` of `node`, keeping its z-order. Returns whether
    /// `node` was present.
    pub fn update_bounds(&mut self, node: NodeId, bounds: Rect) -> bool {
        let Some(entry) = self.entries.get(&node) else {
            return false;
        };
        self.insert(node, bounds, entry.z_order);
        true
    }

    /// Subtracts delta from every indexed origin and rebuilds cell membership. Node IDs and z-order are preserved. A nonfinite shift or resulting origin leaves the index unchanged.
    pub fn rebase_origin(&mut self, delta: Point) -> bool {
        if !delta.x.is_finite() || !delta.y.is_finite() {
            return false;
        }
        if delta == Point::ORIGIN {
            return true;
        }
        let mut rebased = Self::new();
        for (&node, entry) in &self.entries {
            let origin = Point::new(
                entry.bounds.origin.x - delta.x,
                entry.bounds.origin.y - delta.y,
            );
            if !origin.x.is_finite() || !origin.y.is_finite() {
                return false;
            }
            rebased.insert(node, Rect::new(origin, entry.bounds.size), entry.z_order);
        }
        *self = rebased;
        true
    }

    /// Returns whether `node` is present in the index.
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.entries.contains_key(&node)
    }

    /// The number of indexed nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes every entry.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.grid = SpatialGrid::default();
    }

    #[doc = include_str!("../../docs/api/spatial/spatial_index.doc/hits_at.md")]
    #[must_use]
    pub fn hits_at(&self, point: Point) -> Vec<HitTest> {
        let mut hits: Vec<HitTest> = self
            .grid
            .candidates(point)
            .map(|node| (node, &self.entries[&node]))
            .filter(|(_, entry)| entry.bounds.contains(point))
            .map(|(node, entry)| {
                let local_point = Point::new(
                    point.x - entry.bounds.origin.x,
                    point.y - entry.bounds.origin.y,
                );
                HitTest::new(node, local_point, entry.z_order)
            })
            .collect();
        hits.sort_unstable_by_key(|hit| (std::cmp::Reverse(hit.z_order()), hit.node()));
        hits
    }

    /// Returns indexed entries with positive-area overlap with the region, in ascending node ID order. Zero-area entries and rectangles that only touch the region boundary are excluded.
    #[must_use]
    pub fn entries_intersecting(&self, region: Rect) -> Vec<(NodeId, Entry)> {
        if !(region.origin.x < region.right() && region.origin.y < region.bottom()) {
            return Vec::new();
        }
        let overlaps = |entry: &Entry| {
            let bounds = entry.bounds;
            region.origin.x.max(bounds.origin.x) < region.right().min(bounds.right())
                && region.origin.y.max(bounds.origin.y) < region.bottom().min(bounds.bottom())
                && bounds.origin.x < bounds.right()
                && bounds.origin.y < bounds.bottom()
        };
        if let Some(candidates) = self.grid.region_candidates(region) {
            candidates
                .into_iter()
                .filter_map(|node| {
                    let entry = self.entries[&node];
                    overlaps(&entry).then_some((node, entry))
                })
                .collect()
        } else {
            self.entries
                .iter()
                .filter_map(|(&node, &entry)| overlaps(&entry).then_some((node, entry)))
                .collect()
        }
    }

    /// Returns positive-area entries entirely inside the region, including entries that share an edge with it. Results are sorted by node ID.
    #[must_use]
    pub fn entries_contained_by(&self, region: Rect) -> Vec<(NodeId, Entry)> {
        let mut entries = self.entries_intersecting(region);
        entries.retain(|(_, entry)| {
            let bounds = entry.bounds;
            bounds.origin.x >= region.origin.x
                && bounds.origin.y >= region.origin.y
                && bounds.right() <= region.right()
                && bounds.bottom() <= region.bottom()
        });
        entries
    }

    /// Returns the topmost hit at `point`, if any.
    #[must_use]
    pub fn topmost_at(&self, point: Point) -> Option<HitTest> {
        self.hits_at(point).into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(w, h))
    }

    #[test]
    fn new_is_empty() {
        let idx = SpatialIndex::new();
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
        assert_eq!(idx, SpatialIndex::default());
    }

    #[test]
    fn insert_remove_contains() {
        let mut idx = SpatialIndex::new();
        idx.insert(node(1), rect(0.0, 0.0, 10.0, 10.0), 0);
        assert!(idx.contains(node(1)));
        assert_eq!(idx.len(), 1);
        assert!(idx.remove(node(1)));
        assert!(!idx.remove(node(1)));
        assert!(!idx.contains(node(1)));
    }

    #[test]
    fn incremental_insert_replaces() {
        let mut idx = SpatialIndex::new();
        idx.insert(node(1), rect(0.0, 0.0, 10.0, 10.0), 0);
        idx.insert(node(1), rect(5.0, 5.0, 10.0, 10.0), 3);
        assert_eq!(idx.len(), 1);
        // Old bounds no longer hit; new bounds + z_order in effect.
        assert!(idx.hits_at(Point::new(1.0, 1.0)).is_empty());
        let hit = idx.topmost_at(Point::new(6.0, 6.0)).unwrap();
        assert_eq!(hit.z_order(), 3);
    }

    #[test]
    fn update_bounds_moves_entry() {
        let mut idx = SpatialIndex::new();
        idx.insert(node(1), rect(0.0, 0.0, 10.0, 10.0), 2);
        assert!(idx.update_bounds(node(1), rect(20.0, 20.0, 5.0, 5.0)));
        assert!(!idx.update_bounds(node(2), rect(0.0, 0.0, 1.0, 1.0)));
        assert!(idx.hits_at(Point::new(1.0, 1.0)).is_empty());
        let hit = idx.topmost_at(Point::new(21.0, 21.0)).unwrap();
        // z_order preserved across a bounds-only update.
        assert_eq!(hit.z_order(), 2);
    }

    #[test]
    fn hits_at_orders_by_z_order_descending() {
        let mut idx = SpatialIndex::new();
        // Two overlapping rects; node(1) is below, node(2) on top.
        idx.insert(node(1), rect(0.0, 0.0, 10.0, 10.0), 1);
        idx.insert(node(2), rect(0.0, 0.0, 10.0, 10.0), 5);
        let hits = idx.hits_at(Point::new(5.0, 5.0));
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].node(), node(2)); // higher z_order first
        assert_eq!(hits[1].node(), node(1));
        assert_eq!(
            idx.topmost_at(Point::new(5.0, 5.0)).unwrap().node(),
            node(2)
        );
    }

    #[test]
    fn hits_at_tie_break_by_node_id() {
        let mut idx = SpatialIndex::new();
        idx.insert(node(7), rect(0.0, 0.0, 10.0, 10.0), 3);
        idx.insert(node(2), rect(0.0, 0.0, 10.0, 10.0), 3);
        let hits = idx.hits_at(Point::new(5.0, 5.0));
        // Equal z_order → ascending NodeId.
        assert_eq!(hits[0].node(), node(2));
        assert_eq!(hits[1].node(), node(7));
    }

    #[test]
    fn hits_at_translates_local_point() {
        let mut idx = SpatialIndex::new();
        idx.insert(node(1), rect(10.0, 20.0, 30.0, 30.0), 0);
        let hit = idx.topmost_at(Point::new(15.0, 27.0)).unwrap();
        assert_eq!(hit.local_point(), Point::new(5.0, 7.0));
    }

    #[test]
    fn hits_at_excludes_non_containing() {
        let mut idx = SpatialIndex::new();
        idx.insert(node(1), rect(0.0, 0.0, 10.0, 10.0), 0);
        assert!(idx.hits_at(Point::new(50.0, 50.0)).is_empty());
        assert_eq!(idx.topmost_at(Point::new(50.0, 50.0)), None);
    }

    #[test]
    fn grid_finds_candidates_across_cells_and_tracks_mutations() {
        let mut index = SpatialIndex::new();
        index.insert(node(1), rect(-129.0, -1.0, 130.0, 2.0), 0);
        index.insert(node(2), rect(-5000.0, -5000.0, 10000.0, 10000.0), 0);
        index.insert(node(3), rect(512.0, 512.0, 2.0, 2.0), 0);

        for point in [Point::new(-128.0, 0.0), Point::ORIGIN] {
            let candidates: Vec<_> = index.grid.candidates(point).collect();
            assert_eq!(candidates, [node(1), node(2)]);
            assert_eq!(index.topmost_at(point).unwrap().node(), node(1));
        }
        assert_eq!(
            index.topmost_at(Point::new(1.0, 0.0)).unwrap().node(),
            node(2)
        );

        assert!(index.update_bounds(node(2), rect(512.0, 512.0, 2.0, 2.0)));
        index.retain(|node| node != NodeId::from_raw(1));
        assert!(index.grid.candidates(Point::ORIGIN).next().is_none());
        assert!(index.remove(node(2)));
        assert_eq!(
            index.topmost_at(Point::new(512.0, 512.0)).unwrap().node(),
            node(3)
        );
        index.clear();
        assert!(
            index
                .grid
                .candidates(Point::new(512.0, 512.0))
                .next()
                .is_none()
        );
    }

    #[test]
    fn region_queries_deduplicate_and_follow_index_updates() {
        let mut index = SpatialIndex::new();
        index.insert(node(1), rect(-1000.0, -1000.0, 8000.0, 8000.0), 1);
        index.insert(node(2), rect(-5.0, -5.0, 140.0, 140.0), 2);
        index.insert(node(3), rect(130.0, 1.0, 2.0, 2.0), 3);
        index.insert(node(4), rect(1.0, 1.0, 0.0, 2.0), 4);
        index.insert(node(5), rect(60.0, 180.0, 2.0, 2.0), 5);

        let region = rect(-1.0, -1.0, 131.0, 131.0);
        let ids = |index: &SpatialIndex, region| {
            index
                .entries_intersecting(region)
                .into_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&index, region), [node(1), node(2)]);
        assert_eq!(
            ids(&index, rect(-10_000.0, -10_000.0, 20_000.0, 20_000.0)),
            [node(1), node(2), node(3), node(5)]
        );
        assert!(ids(&index, rect(0.0, 0.0, 0.0, 10.0)).is_empty());
        assert!(ids(&index, rect(f32::NAN, 0.0, 10.0, 10.0)).is_empty());

        assert!(index.remove(node(1)));
        assert!(index.update_bounds(node(2), rect(300.0, 300.0, 2.0, 2.0)));
        assert!(ids(&index, region).is_empty());
    }

    #[test]
    fn rebasing_updates_point_and_region_queries_atomically() {
        let mut index = SpatialIndex::new();
        index.insert(node(1), rect(1000.0, -600.0, 20.0, 20.0), 3);
        index.insert(node(2), rect(1005.0, -595.0, 20.0, 20.0), 4);

        assert!(index.rebase_origin(Point::new(1000.0, -600.0)));
        assert!(index.hits_at(Point::new(1010.0, -590.0)).is_empty());
        let hit = index.topmost_at(Point::new(10.0, 10.0)).unwrap();
        assert_eq!(hit.node(), node(2));
        assert_eq!(hit.local_point(), Point::new(5.0, 5.0));
        let entries = index.entries_intersecting(rect(0.0, 0.0, 30.0, 30.0));
        assert_eq!(
            entries,
            [
                (node(1), Entry::new(rect(0.0, 0.0, 20.0, 20.0), 3)),
                (node(2), Entry::new(rect(5.0, 5.0, 20.0, 20.0), 4))
            ]
        );

        index.insert(node(3), rect(f32::MAX, 0.0, 20.0, 20.0), 0);
        let previous = index.clone();
        assert!(!index.rebase_origin(Point::new(-f32::MAX, 0.0)));
        assert_eq!(index, previous);
    }

    #[test]
    fn clear_empties() {
        let mut idx = SpatialIndex::new();
        idx.insert(node(1), rect(0.0, 0.0, 10.0, 10.0), 0);
        idx.clear();
        assert!(idx.is_empty());
    }
}
