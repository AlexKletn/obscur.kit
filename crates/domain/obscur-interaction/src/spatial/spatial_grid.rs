//! Grid used to find candidates for point queries.

use std::collections::{BTreeSet, HashMap};

use obscur_geometry::{Point, Rect};
use obscur_identity::NodeId;

const CELL_SIZE: f64 = 128.0;
const MAX_CELLS_PER_ENTRY: usize = 64;

type Cell = (i32, i32);

/// Indexes bounded rectangles in fixed-size cells and tracks wide rectangles separately.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct SpatialGrid {
    cells: HashMap<Cell, BTreeSet<NodeId>>,
    overflow: BTreeSet<NodeId>,
}

impl SpatialGrid {
    /// Adds the node to the cells covered by its bounds or to the overflow set.
    pub(super) fn insert(&mut self, node: NodeId, bounds: Rect) {
        if let Some(cells) = covered_cells(bounds, MAX_CELLS_PER_ENTRY) {
            for cell in cells {
                self.cells.entry(cell).or_default().insert(node);
            }
        } else {
            self.overflow.insert(node);
        }
    }

    /// Removes the node using its indexed bounds.
    pub(super) fn remove(&mut self, node: NodeId, bounds: Rect) {
        if let Some(cells) = covered_cells(bounds, MAX_CELLS_PER_ENTRY) {
            for cell in cells {
                if let std::collections::hash_map::Entry::Occupied(mut slot) =
                    self.cells.entry(cell)
                {
                    slot.get_mut().remove(&node);
                    if slot.get().is_empty() {
                        slot.remove();
                    }
                }
            }
        } else {
            self.overflow.remove(&node);
        }
    }

    /// Yields the nodes in the point’s cell and the overflow set.
    pub(super) fn candidates(&self, point: Point) -> impl Iterator<Item = NodeId> + '_ {
        cell_at(point.x, point.y)
            .and_then(|cell| self.cells.get(&cell))
            .into_iter()
            .flatten()
            .chain(&self.overflow)
            .copied()
    }

    /// Collects candidates from the covered grid cells and the overflow set. Wide regions fall back to an entry scan.
    pub(super) fn region_candidates(&self, region: Rect) -> Option<BTreeSet<NodeId>> {
        let cells = covered_cells(region, self.cells.len().max(MAX_CELLS_PER_ENTRY))?;
        Some(
            cells
                .filter_map(|cell| self.cells.get(&cell))
                .flatten()
                .chain(&self.overflow)
                .copied()
                .collect(),
        )
    }
}

fn cell_at(x: f32, y: f32) -> Option<Cell> {
    fn index(value: f32) -> Option<i32> {
        let cell = (f64::from(value) / CELL_SIZE).floor();
        (f64::from(i32::MIN)..=f64::from(i32::MAX))
            .contains(&cell)
            .then_some(cell as i32)
    }

    Some((index(x)?, index(y)?))
}

fn covered_cells(bounds: Rect, limit: usize) -> Option<impl Iterator<Item = Cell>> {
    let (left, top) = cell_at(bounds.origin.x, bounds.origin.y)?;
    let (right, bottom) = cell_at(bounds.right(), bounds.bottom())?;
    let columns = u64::try_from(i64::from(right) - i64::from(left) + 1).ok()?;
    let rows = u64::try_from(i64::from(bottom) - i64::from(top) + 1).ok()?;
    if columns.saturating_mul(rows) > limit as u64 {
        return None;
    }
    Some((top..=bottom).flat_map(move |y| (left..=right).map(move |x| (x, y))))
}
