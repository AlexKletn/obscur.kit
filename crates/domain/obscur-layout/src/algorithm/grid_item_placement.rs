#![doc = include_str!("grid_item_placement.doc/_module.md")]

use super::LayoutChild;
use super::grid::GridAutoFlow;

#[doc = include_str!("grid_item_placement.doc/PlacedItems.md")]
pub(super) struct PlacedItems {
    #[doc = include_str!("grid_item_placement.doc/col.md")]
    pub(super) col: Vec<usize>,
    #[doc = include_str!("grid_item_placement.doc/row.md")]
    pub(super) row: Vec<usize>,
    #[doc = include_str!("grid_item_placement.doc/col_span.md")]
    pub(super) col_span: Vec<usize>,
    #[doc = include_str!("grid_item_placement.doc/row_span.md")]
    pub(super) row_span: Vec<usize>,
    #[doc = include_str!("grid_item_placement.doc/n_cols.md")]
    pub(super) n_cols: usize,
    #[doc = include_str!("grid_item_placement.doc/n_rows.md")]
    pub(super) n_rows: usize,
}

impl PlacedItems {
    #[doc = include_str!("grid_item_placement.doc/resolve.md")]
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
        for child in children {
            let placement = child.style().grid_placement();
            let (ca, cs) = placement.resolve_column(templ_cols_len);
            let (ra, rs) = placement.resolve_row(templ_rows_len);
            col_anchor.push(ca);
            col_span.push(cs.max(1));
            row_anchor.push(ra);
            row_span.push(rs.max(1));
        }

        // Column count: the explicit tracks, grown to cover any definite column
        // line that overflows the template and any span wider than it (implicit
        // auto columns). With all-`Auto` placement (every span 1, no definite
        // line) this is exactly the template width — the historical grid.
        let mut n_cols = templ_cols_len.max(1);
        for i in 0..count {
            if let Some(c) = col_anchor[i] {
                n_cols = n_cols.max(c + col_span[i]);
            }
            n_cols = n_cols.max(col_span[i]);
        }

        // Row-wrap bound for **column** auto-flow: the explicit rows, grown to
        // cover any definite row line / row span (the row analogue of `n_cols`).
        // Only consulted when the container flows column-major.
        let mut n_rows_flow = templ_rows_len.max(1);
        for i in 0..count {
            if let Some(r) = row_anchor[i] {
                n_rows_flow = n_rows_flow.max(r + row_span[i]);
            }
            n_rows_flow = n_rows_flow.max(row_span[i]);
        }

        // Two-phase placement into an occupancy grid. Phase A — items with a
        // definite column and/or row: honour the definite axis, auto-flow any
        // missing axis. Phase B — fully-auto items, placed per the container's
        // `grid-auto-flow`: row- vs column-major (the iteration axis) and sparse
        // vs dense (a forward-only cursor vs a from-the-origin search that
        // backfills earlier holes). With the default `Row` (sparse) flow and
        // all-`Auto` placement Phase A is empty and Phase B fills cell `i` in
        // row-major order — byte-identical to the previous auto-placement.
        let mut occ: Vec<Vec<bool>> = Vec::new();
        let mut place_col = vec![0_usize; count];
        let mut place_row = vec![0_usize; count];
        for i in 0..count {
            let (ca, ra) = (col_anchor[i], row_anchor[i]);
            if ca.is_none() && ra.is_none() {
                continue; // fully auto → Phase B
            }
            let (cspan, rspan) = (col_span[i], row_span[i]);
            let (c0, r0) = match (ca, ra) {
                (Some(c), Some(r)) => (c, r),
                (Some(c), None) => (c, first_row(&occ, c, cspan, rspan, n_cols)),
                (None, Some(r)) => (first_col(&occ, r, cspan, rspan, n_cols), r),
                (None, None) => unreachable!(),
            };
            mark_block(&mut occ, r0, rspan, c0, cspan, n_cols);
            place_col[i] = c0;
            place_row[i] = r0;
        }
        // Phase B: the auto-placement cursor. `is_dense` searches from the origin
        // every item (backfilling holes); the sparse cursor only moves forward.
        let (mut cur_major, mut cur_minor) = (0_usize, 0_usize);
        for i in 0..count {
            if col_anchor[i].is_some() || row_anchor[i].is_some() {
                continue; // placed in Phase A
            }
            let (cspan, rspan) = (col_span[i], row_span[i]);
            let (c0, r0) = if flow.is_column() {
                // Column-major: `cur_major` is the column, `cur_minor` the row.
                let (sc, sr) = if flow.is_dense() {
                    (0, 0)
                } else {
                    (cur_major, cur_minor)
                };
                first_fit_col_major(&occ, cspan, rspan, n_rows_flow, sc, sr)
            } else if flow.is_dense() {
                first_fit(&occ, cspan, rspan, n_cols)
            } else {
                // Row-major sparse: `cur_major` is the row, `cur_minor` the column.
                first_fit_from(&occ, cspan, rspan, n_cols, cur_major, cur_minor)
            };
            mark_block(&mut occ, r0, rspan, c0, cspan, n_cols);
            place_col[i] = c0;
            place_row[i] = r0;
            if !flow.is_dense() {
                (cur_major, cur_minor) = if flow.is_column() { (c0, r0) } else { (r0, c0) };
            }
        }
        // Column-flow items may land past the explicit column count (implicit
        // columns); grow `n_cols` to cover them. For row flow every placed column
        // is already within `n_cols`, so this is a no-op (byte-identical).
        for i in 0..count {
            n_cols = n_cols.max(place_col[i] + col_span[i]);
        }

        // Total rows the grid spans (implicit auto rows past the template).
        let n_rows = (0..count)
            .map(|i| place_row[i] + row_span[i])
            .max()
            .unwrap_or(0);

        Self {
            col: place_col,
            row: place_row,
            col_span,
            row_span,
            n_cols,
            n_rows,
        }
    }
}

#[doc = include_str!("grid_item_placement.doc/block_free.md")]
fn block_free(
    occ: &[Vec<bool>],
    row: usize,
    rspan: usize,
    col: usize,
    cspan: usize,
    n_cols: usize,
) -> bool {
    if col + cspan > n_cols {
        return false;
    }
    for line in occ.iter().skip(row).take(rspan) {
        for &cell in line.iter().skip(col).take(cspan) {
            if cell {
                return false;
            }
        }
    }
    true
}

#[doc = include_str!("grid_item_placement.doc/mark_block.md")]
fn mark_block(
    occ: &mut Vec<Vec<bool>>,
    row: usize,
    rspan: usize,
    col: usize,
    cspan: usize,
    n_cols: usize,
) {
    let width = n_cols.max(col + cspan);
    while occ.len() < row + rspan {
        occ.push(vec![false; width]);
    }
    for line in occ.iter_mut() {
        if line.len() < width {
            line.resize(width, false);
        }
    }
    for line in occ.iter_mut().skip(row).take(rspan) {
        for cell in line.iter_mut().skip(col).take(cspan) {
            *cell = true;
        }
    }
}

#[doc = include_str!("grid_item_placement.doc/first_row.md")]
fn first_row(occ: &[Vec<bool>], col: usize, cspan: usize, rspan: usize, n_cols: usize) -> usize {
    let mut row = 0;
    while !block_free(occ, row, rspan, col, cspan, n_cols) {
        row += 1;
    }
    row
}

#[doc = include_str!("grid_item_placement.doc/first_col.md")]
fn first_col(occ: &[Vec<bool>], row: usize, cspan: usize, rspan: usize, n_cols: usize) -> usize {
    if cspan > n_cols {
        return 0;
    }
    (0..=(n_cols - cspan))
        .find(|&col| block_free(occ, row, rspan, col, cspan, n_cols))
        .unwrap_or(0)
}

#[doc = include_str!("grid_item_placement.doc/first_fit.md")]
fn first_fit(occ: &[Vec<bool>], cspan: usize, rspan: usize, n_cols: usize) -> (usize, usize) {
    if cspan > n_cols {
        return (0, 0);
    }
    let mut row = 0;
    loop {
        if let Some(col) =
            (0..=(n_cols - cspan)).find(|&col| block_free(occ, row, rspan, col, cspan, n_cols))
        {
            return (col, row);
        }
        row += 1;
    }
}

#[doc = include_str!("grid_item_placement.doc/block_is_free.md")]
fn block_is_free(occ: &[Vec<bool>], row: usize, rspan: usize, col: usize, cspan: usize) -> bool {
    for line in occ.iter().skip(row).take(rspan) {
        for c in col..col + cspan {
            if line.get(c).copied().unwrap_or(false) {
                return false;
            }
        }
    }
    true
}

#[doc = include_str!("grid_item_placement.doc/first_fit_from.md")]
fn first_fit_from(
    occ: &[Vec<bool>],
    cspan: usize,
    rspan: usize,
    n_cols: usize,
    start_row: usize,
    start_col: usize,
) -> (usize, usize) {
    if cspan > n_cols {
        return (0, start_row);
    }
    let hi = n_cols - cspan;
    let mut row = start_row;
    let mut col_lo = start_col;
    loop {
        if col_lo <= hi
            && let Some(col) =
                (col_lo..=hi).find(|&col| block_free(occ, row, rspan, col, cspan, n_cols))
        {
            return (col, row);
        }
        row += 1;
        col_lo = 0;
    }
}

#[doc = include_str!("grid_item_placement.doc/first_fit_col_major.md")]
fn first_fit_col_major(
    occ: &[Vec<bool>],
    cspan: usize,
    rspan: usize,
    n_rows: usize,
    start_col: usize,
    start_row: usize,
) -> (usize, usize) {
    if rspan > n_rows {
        return (start_col, 0);
    }
    let hi = n_rows - rspan;
    let mut col = start_col;
    let mut row_lo = start_row;
    loop {
        if row_lo <= hi
            && let Some(row) = (row_lo..=hi).find(|&row| block_is_free(occ, row, rspan, col, cspan))
        {
            return (col, row);
        }
        col += 1;
        row_lo = 0;
    }
}
