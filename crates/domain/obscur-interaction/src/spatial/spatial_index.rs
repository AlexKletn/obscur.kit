//! The [`SpatialIndex`] — the incremental, z-order-aware index backing hit
//! testing (spec §4.7).

use std::collections::BTreeMap;

use obscur_geometry::{Point, Rect};
use obscur_identity::NodeId;

use super::HitTest;

/// A single indexed node: its visual `bounds` and `z_order` (spec §4.7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    /// The node's already-resolved visual bounds (transform/clip applied
    /// upstream).
    pub bounds: Rect,
    /// The node's z-order; higher values are drawn on top.
    pub z_order: i32,
}

impl Entry {
    /// Constructs an entry from resolved `bounds` and a `z_order`.
    #[must_use]
    pub const fn new(bounds: Rect, z_order: i32) -> Self {
        Self { bounds, z_order }
    }
}

/// The incremental spatial index used for hit testing (spec §4.7).
///
/// The index stores each node's resolved visual [`bounds`](Entry::bounds) and
/// [`z_order`](Entry::z_order). It is maintained *incrementally* — nodes are
/// inserted, removed, and re-bounded individually rather than rebuilt — and is
/// backed by a [`BTreeMap`] keyed by [`NodeId`] for deterministic iteration.
///
/// Queries return [`HitTest`] records ordered topmost-first (descending
/// z-order, ties broken by `NodeId`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpatialIndex {
    entries: BTreeMap<NodeId, Entry>,
}

impl SpatialIndex {
    /// Creates an empty index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Inserts or replaces the entry for `node` — the incremental update.
    pub fn insert(&mut self, node: NodeId, bounds: Rect, z_order: i32) {
        self.entries.insert(node, Entry::new(bounds, z_order));
    }

    /// Removes `node` from the index, returning whether it was present.
    pub fn remove(&mut self, node: NodeId) -> bool {
        self.entries.remove(&node).is_some()
    }

    /// Updates only the `bounds` of `node`, keeping its z-order. Returns whether
    /// `node` was present.
    pub fn update_bounds(&mut self, node: NodeId, bounds: Rect) -> bool {
        match self.entries.get_mut(&node) {
            Some(entry) => {
                entry.bounds = bounds;
                true
            }
            None => false,
        }
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
    }

    /// Returns every entry whose bounds contain `point`, ordered topmost-first.
    ///
    /// Ordering is by descending [`z_order`](Entry::z_order); ties are broken
    /// deterministically by ascending [`NodeId`]. Each result's
    /// [`local_point`](HitTest::local_point) is `point` translated into the
    /// entry's local space (`point - bounds.origin`).
    #[must_use]
    pub fn hits_at(&self, point: Point) -> Vec<HitTest> {
        let mut hits: Vec<HitTest> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.bounds.contains(point))
            .map(|(node, entry)| {
                let local_point = Point::new(
                    point.x - entry.bounds.origin.x,
                    point.y - entry.bounds.origin.y,
                );
                HitTest::new(*node, local_point, entry.z_order)
            })
            .collect();
        // Topmost-first: higher z_order wins; ties broken by NodeId ascending.
        // The source iteration is already NodeId-ascending, so a stable sort by
        // descending z_order preserves that as the tie-break.
        hits.sort_by_key(|hit| std::cmp::Reverse(hit.z_order()));
        hits
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
        assert_eq!(idx.topmost_at(Point::new(5.0, 5.0)).unwrap().node(), node(2));
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
    fn clear_empties() {
        let mut idx = SpatialIndex::new();
        idx.insert(node(1), rect(0.0, 0.0, 10.0, 10.0), 0);
        idx.clear();
        assert!(idx.is_empty());
    }
}
