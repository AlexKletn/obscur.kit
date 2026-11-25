//! Tracks the visible set for an application-managed spatial scene.

use std::collections::BTreeMap;

use obscur_geometry::CoordinateMapper;
use obscur_geometry::space::{PhysicalSpace, RectIn, WorldSpace};
use obscur_identity::NodeId;

use super::{DetailLevels, MaterializationChange, SpatialIndex};

/// Retains object identity across successive queries in world coordinates.
#[derive(Debug, Clone, Default)]
pub struct SpatialMaterialization {
    active: BTreeMap<NodeId, Option<usize>>,
}

impl SpatialMaterialization {
    /// Reconciles the visible set with the current index and world region. Call after the camera or indexed bounds change.
    pub fn update(
        &mut self,
        index: &SpatialIndex,
        region: RectIn<WorldSpace>,
    ) -> MaterializationChange {
        let next = index
            .entries_intersecting(region.rect())
            .into_iter()
            .map(|(id, _)| (id, None))
            .collect();
        self.replace_active(next)
    }

    /// Updates membership and reports detail levels for newly visible objects or objects whose level changed. The level uses the longer side of the projected bounds in physical pixels.
    pub fn update_with_detail(
        &mut self,
        index: &SpatialIndex,
        region: RectIn<WorldSpace>,
        physical: &CoordinateMapper<WorldSpace, PhysicalSpace>,
        levels: &DetailLevels,
    ) -> (MaterializationChange, Vec<(NodeId, usize)>) {
        let mut detail_changes = Vec::new();
        let next = index
            .entries_intersecting(region.rect())
            .into_iter()
            .map(|(id, entry)| {
                let size = physical.map_extent(entry.bounds.size);
                let level = levels.level_for(size.width.max(size.height));
                if self.active.get(&id).copied().flatten() != Some(level) {
                    detail_changes.push((id, level));
                }
                (id, Some(level))
            })
            .collect();
        (self.replace_active(next), detail_changes)
    }

    fn replace_active(&mut self, next: BTreeMap<NodeId, Option<usize>>) -> MaterializationChange {
        let change = MaterializationChange {
            entered: next
                .keys()
                .copied()
                .filter(|id| !self.active.contains_key(id))
                .collect(),
            exited: self
                .active
                .keys()
                .copied()
                .filter(|id| !next.contains_key(id))
                .collect(),
        };
        self.active = next;
        change
    }

    /// Iterates the current visible objects in ascending node ID order.
    pub fn active(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.active.keys().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Rect, Size};

    #[test]
    fn updates_preserve_overlapping_entities_and_remove_deleted_ones() {
        let mut index = SpatialIndex::new();
        let region = |x| Rect::new(Point::new(x, 0.0), Size::new(20.0, 10.0));
        let [a, b, c] = [1, 2, 3].map(NodeId::from_raw);
        for (node, x) in [(a, 0.0), (b, 20.0), (c, 40.0)] {
            index.insert(node, region(x), 0);
        }
        let mut state = SpatialMaterialization::default();
        let first = state.update(&index, RectIn::new(region(10.0)));
        assert_eq!(first.entered, [a, b]);
        assert!(first.exited.is_empty());
        assert!(state.update(&index, RectIn::new(region(10.0))).is_empty());

        let moved = state.update(&index, RectIn::new(region(30.0)));
        assert_eq!(moved.entered, [c]);
        assert_eq!(moved.exited, [a]);
        assert_eq!(state.active().collect::<Vec<_>>(), [b, c]);

        assert!(index.remove(b));
        let deleted = state.update(&index, RectIn::new(region(30.0)));
        assert!(deleted.entered.is_empty());
        assert_eq!(deleted.exited, [b]);
        assert_eq!(state.active().collect::<Vec<_>>(), [c]);
    }

    #[test]
    fn detail_changes_with_scale_while_materialized_identity_stays_stable() {
        use obscur_geometry::AffineTransform;

        let node = NodeId::from_raw(1);
        let bounds = Rect::new(Point::ORIGIN, Size::new(20.0, 10.0));
        let region = RectIn::new(bounds);
        let mut index = SpatialIndex::new();
        index.insert(node, bounds, 0);
        assert!(DetailLevels::new(vec![48.0, 32.0]).is_none());
        let levels = DetailLevels::new(vec![32.0, 64.0]).unwrap();
        let mut state = SpatialMaterialization::default();
        let identity = CoordinateMapper::new(AffineTransform::IDENTITY);
        let (membership, update) = state.update_with_detail(&index, region, &identity, &levels);
        assert_eq!(membership.entered, [node]);
        assert_eq!(update, [(node, 0)]);

        let zoomed = CoordinateMapper::new(
            AffineTransform::translate(1e10, -1e10).multiply(AffineTransform::scale(3.2, 3.2)),
        );
        let (membership, update) = state.update_with_detail(&index, region, &zoomed, &levels);
        assert!(membership.is_empty());
        assert_eq!(update, [(node, 2)]);
        let (membership, update) = state.update_with_detail(&index, region, &zoomed, &levels);
        assert!(membership.is_empty() && update.is_empty());

        index.remove(node);
        let (membership, update) = state.update_with_detail(&index, region, &zoomed, &levels);
        assert_eq!(membership.exited, [node]);
        assert!(update.is_empty());
    }
}
