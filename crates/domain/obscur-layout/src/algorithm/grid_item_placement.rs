#![doc = include_str!("../../docs/api/algorithm/grid_item_placement.doc/_module.md")]

use std::collections::HashMap;

use obscur_identity::NodeId;

use super::LayoutChild;
use super::grid::{GridAutoFlow, preceding_tracks, resolve_axis};
use super::layout_child::ordered_child_indices;

#[doc = include_str!("../../docs/api/algorithm/grid_item_placement.doc/PlacedItems.md")]
pub(super) struct PlacedItems {
    /// Each item's start column.
    pub(super) col: Vec<usize>,
    /// Each item's start row.
    pub(super) row: Vec<usize>,
    /// Each item's column span (at least `1`).
    pub(super) col_span: Vec<usize>,
    /// Each item's row span (at least `1`).
    pub(super) row_span: Vec<usize>,
    /// Total columns the grid spans, explicit and implicit.
    pub(super) n_cols: usize,
    /// Total rows the grid spans, explicit and implicit.
    pub(super) n_rows: usize,
    pub(super) preceding_cols: usize,
    pub(super) preceding_rows: usize,
    pub(super) limited_item: Option<NodeId>,
}

impl PlacedItems {
    #[doc = include_str!("../../docs/api/algorithm/grid_item_placement.doc/resolve.md")]
    pub(super) fn resolve(
        children: &[LayoutChild],
        flow: GridAutoFlow,
        templ_cols_len: usize,
        templ_rows_len: usize,
    ) -> Self {
        let count = children.len();

        // Resolve each child's line-based placement into a definite/auto anchor
        // plus its column/row span (spec §4.13, a subset). A default placement is
        // all-`Auto` → `(None, 1)` on both axes: pure row-major auto-placement.
        let mut col_anchor = Vec::with_capacity(count);
        let mut col_span = Vec::with_capacity(count);
        let mut row_anchor = Vec::with_capacity(count);
        let mut row_span = Vec::with_capacity(count);
        let (preceding_cols, preceding_rows) =
            children.iter().fold((0, 0), |(cols, rows), child| {
                let placement = child.style().grid_placement();
                (
                    cols.max(preceding_tracks(
                        placement.column_start,
                        placement.column_end,
                        templ_cols_len,
                    )),
                    rows.max(preceding_tracks(
                        placement.row_start,
                        placement.row_end,
                        templ_rows_len,
                    )),
                )
            });
        let mut limited_item = None;
        for child in children {
            let placement = child.style().grid_placement();
            let ((ca, cs), col_limited) = resolve_axis(
                placement.column_start,
                placement.column_end,
                templ_cols_len,
                preceding_cols,
            );
            let ((ra, rs), row_limited) = resolve_axis(
                placement.row_start,
                placement.row_end,
                templ_rows_len,
                preceding_rows,
            );
            if col_limited || row_limited {
                limited_item.get_or_insert(child.node());
            }
            col_anchor.push(ca);
            col_span.push(cs.max(1));
            row_anchor.push(ra);
            row_span.push(rs.max(1));
        }

        let templ_cols_len = templ_cols_len + preceding_cols;
        let templ_rows_len = templ_rows_len + preceding_rows;

        // Use one placement algorithm for both directions. In column flow
        // its rows are physical columns and its columns are physical rows.
        let mut column_count = templ_cols_len;
        if flow.is_column() {
            std::mem::swap(&mut col_anchor, &mut row_anchor);
            std::mem::swap(&mut col_span, &mut row_span);
            column_count = templ_rows_len;
        }

        let mut occupied = Vec::new();
        let mut place_col = vec![0; count];
        let mut place_row = vec![0; count];

        // Fully definite items can overlap one another, but reserve their
        // cells before any item searches for a free position.
        for i in 0..count {
            if let (Some(col), Some(row)) = (col_anchor[i], row_anchor[i]) {
                place_col[i] = col;
                place_row[i] = row;
                mark_block(&mut occupied, row, row_span[i], col, col_span[i]);
            }
        }

        let order = ordered_child_indices(children);
        let item_index = |i| order.as_ref().map_or(i, |indices| indices[i]);

        // Items locked to a row may grow the grid in the other direction.
        let mut locked_row_end = HashMap::new();
        for i in (0..count).map(item_index) {
            if let (None, Some(row)) = (col_anchor[i], row_anchor[i]) {
                let mut col = locked_row_end.get(&row).copied().unwrap_or(0);
                while !block_free(&occupied, row, row_span[i], col, col_span[i]) {
                    col += 1;
                }
                if !flow.is_dense() {
                    locked_row_end.insert(row, col + col_span[i]);
                }
                place_col[i] = col;
                place_row[i] = row;
                mark_block(&mut occupied, row, row_span[i], col, col_span[i]);
            }
        }

        let mut n_cols = column_count.max(1);
        for i in 0..count {
            let col = if row_anchor[i].is_some() {
                place_col[i]
            } else {
                col_anchor[i].unwrap_or(0)
            };
            n_cols = n_cols.max(col + col_span[i]);
        }

        // Place the remaining items in order-modified source order. Sparse flows keep the
        // cursor forward-only while dense flows restart each search at zero.
        let (mut row, mut col) = (0, 0);
        for i in (0..count).map(item_index) {
            if row_anchor[i].is_some() {
                continue;
            }
            if flow.is_dense() {
                (row, col) = (0, 0);
            }
            let (cspan, rspan) = (col_span[i], row_span[i]);
            if let Some(fixed_col) = col_anchor[i] {
                row += usize::from(fixed_col < col);
                col = fixed_col;
                while !block_free(&occupied, row, rspan, col, cspan) {
                    row += 1;
                }
            } else {
                (col, row) = first_fit_from(&occupied, cspan, rspan, n_cols, row, col);
            }
            place_col[i] = col;
            place_row[i] = row;
            mark_block(&mut occupied, row, rspan, col, cspan);
        }

        if flow.is_column() {
            std::mem::swap(&mut place_col, &mut place_row);
            std::mem::swap(&mut col_span, &mut row_span);
            n_cols = (0..count)
                .map(|i| place_col[i] + col_span[i])
                .fold(templ_cols_len.max(1), usize::max);
        }

        let n_rows = (0..count)
            .map(|i| place_row[i] + row_span[i])
            .fold(templ_rows_len, usize::max);

        Self {
            col: place_col,
            row: place_row,
            col_span,
            row_span,
            n_cols,
            n_rows,
            preceding_cols,
            preceding_rows,
            limited_item,
        }
    }
}

/// Checks whether a block is free. Cells beyond the stored occupancy are free.
fn block_free(occ: &[Vec<bool>], row: usize, rspan: usize, col: usize, cspan: usize) -> bool {
    occ.iter()
        .skip(row)
        .take(rspan)
        .all(|line| !line.iter().skip(col).take(cspan).any(|&cell| cell))
}

/// Marks a block occupied, extending the occupancy rows touched by the block.
fn mark_block(occ: &mut Vec<Vec<bool>>, row: usize, rspan: usize, col: usize, cspan: usize) {
    if occ.len() < row + rspan {
        occ.resize_with(row + rspan, Vec::new);
    }
    let end = col + cspan;
    for line in &mut occ[row..row + rspan] {
        if line.len() < end {
            line.resize(end, false);
        }
        line[col..end].fill(true);
    }
}

/// Finds the first free `(col, row)` in row-major order at or after the cursor.
/// The current row starts at `start_col`. Later rows start at column zero.
/// The column bound must fit at least one item span.
fn first_fit_from(
    occ: &[Vec<bool>],
    cspan: usize,
    rspan: usize,
    n_cols: usize,
    start_row: usize,
    start_col: usize,
) -> (usize, usize) {
    let hi = n_cols - cspan;
    let mut row = start_row;
    let mut col_lo = start_col;
    loop {
        if col_lo <= hi
            && let Some(col) = (col_lo..=hi).find(|&col| block_free(occ, row, rspan, col, cspan))
        {
            return (col, row);
        }
        row += 1;
        col_lo = 0;
    }
}
