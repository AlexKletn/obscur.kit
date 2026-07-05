//! `GridLayout`: a real CSS-grid track-sizing formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{LayoutBox, LayoutStyle};

use super::grid::{GridTemplate, TrackSize};
use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// CSS-grid layout with real per-track sizing (spec §4.1, §4.2).
///
/// Children flow **row-major** into a grid described by a [`GridTemplate`]:
/// child `0` fills column `0` of row `0`, child `1` column `1`, and so on,
/// wrapping to a new row once every column in the current row is filled. Each
/// column and row is sized independently by its [`TrackSize`] — a fixed pixel
/// length, a flexible `fr` fraction, a content-sized `auto`, or a
/// `minmax(min, max)` clamp.
///
/// # Configuration
///
/// The grid carries a [`GridTemplate`] (the column and row track lists) plus a
/// horizontal `column_gap` and vertical `row_gap`. The uniform constructor
/// [`GridLayout::new`] builds an *N equal columns* grid by using `N` `1fr`
/// columns and all-implicit auto rows, so a plain `new(n)` still lays out `n`
/// equal-width columns exactly as before.
///
/// # Algorithm
///
/// 1. **Content area** — the container's own explicit `width`/`height` (resolved
///    against `constraints.max`) gives the content width/height; an `Auto` axis
///    falls back to `constraints.max` minus `padding` (each clamped ≥ 0). Tracks,
///    `fr` distribution, and percentages all resolve against this extent.
/// 2. **Column track sizing** — with `n = columns.len()` and
///    `available = content_width - (n - 1) * column_gap`, each column resolves to:
///    `Fixed(px)` → `px`; `Auto` → the largest measured content width among the
///    items auto-placed in it (measured under loose constraints); `Fr(f)` → a
///    share `f / Σfr` of the space left after the fixed + auto tracks; and
///    `MinMax(min, max)` → the `max` function resolved, then floored at the `min`
///    function (so `minmax(px, fr)` grows flexibly from `px` up). `fr` space is
///    distributed exactly like [`FlexLayout`](super::FlexLayout) shares free
///    space among grow factors.
/// 3. **Row track sizing** — the same resolution over the explicit `rows` using
///    `content_height` and `row_gap`. Rows past the explicit list are *implicit*
///    and sized as `Auto` (as tall as the tallest item placed in them).
/// 4. **Placement** — each child carries a [`GridPlacement`](super::GridPlacement)
///    (its `grid-column` / `grid-row` line pair). Placement runs in two phases:
///    *Phase A* honours every item with a definite column and/or row line — a
///    fully-definite item lands at its cell, a half-definite item locks the
///    definite axis and auto-flows the other — marking each `col-span × row-span`
///    block occupied. *Phase B* then places the remaining fully-auto items
///    row-major by **first-fit**: each drops into the first free cell (skipping
///    occupied cells, wrapping a too-wide span to the next row) that fits its
///    span. Definite lines past the template add implicit `Auto` columns; rows
///    past the template are implicit `Auto`. With every child left at the default
///    all-`Auto` placement this reduces exactly to `(i % n_cols, i / n_cols)`.
/// 5. **Sizing** — an item spanning `s` columns is `Σ(spanned column widths) +
///    (s - 1) * column_gap` wide (analogously for a row span with `row_gap`); a
///    spanning item attributes its content size to its first spanned track. It is
///    measured with its width fixed to that span; a single content-sized (`Auto`
///    / implicit) row keeps its own measured height, otherwise it fills its
///    spanned row tracks. It is placed at
///    `(padding.left + Σ(prior column widths) + start_col * column_gap,
///      padding.top  + Σ(prior row heights)  + start_row * row_gap)`.
/// 6. **Border box** — an explicit container axis sizes to that content extent
///    plus `padding`; an `Auto` axis sizes to the bounding box of the placed
///    children plus `padding`. The result is clamped into `constraints`.
///
/// # Out of scope (later steps)
///
/// This does real track sizing plus line-based placement and spanning, but is
/// still a subset: no *dense* auto-placement packing (placement is first-fit,
/// never backfilling earlier holes), no named or negative grid lines, and no
/// alignment of an item within its track. Half-definite items always auto-flow
/// the missing axis. Those belong to later work; the CSS parsing and runtime
/// wiring of placement is a later step.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridLayout {
    /// The explicit column/row track template.
    template: GridTemplate,
    /// Horizontal gap between adjacent columns, in logical pixels.
    column_gap: f32,
    /// Vertical gap between adjacent rows, in logical pixels.
    row_gap: f32,
}

impl GridLayout {
    /// Creates a uniform grid of `columns` equal-width columns and no gaps.
    ///
    /// `columns` is clamped to a minimum of `1`, so a zero column count still
    /// produces a valid single-column grid. Each column is a `1fr` track, so the
    /// columns share the content width equally — the historical uniform-grid
    /// behaviour — and every row is implicit (auto, content-sized).
    #[must_use]
    pub fn new(columns: usize) -> Self {
        let n = columns.max(1);
        Self {
            template: GridTemplate::columns_only(vec![TrackSize::Fr(1.0); n]),
            column_gap: 0.0,
            row_gap: 0.0,
        }
    }

    /// Creates a grid from an explicit [`GridTemplate`] and the two gaps.
    #[must_use]
    pub fn with_template(template: GridTemplate, column_gap: f32, row_gap: f32) -> Self {
        Self {
            template,
            column_gap,
            row_gap,
        }
    }

    /// Returns a copy with the horizontal `column_gap` replaced.
    #[must_use]
    pub fn with_column_gap(mut self, column_gap: f32) -> Self {
        self.column_gap = column_gap;
        self
    }

    /// Returns a copy with the vertical `row_gap` replaced.
    #[must_use]
    pub fn with_row_gap(mut self, row_gap: f32) -> Self {
        self.row_gap = row_gap;
        self
    }

    /// The grid's track template.
    #[must_use]
    pub const fn template(&self) -> &GridTemplate {
        &self.template
    }

    /// The number of columns (always at least `1`).
    #[must_use]
    pub fn columns(&self) -> usize {
        self.template.columns().len().max(1)
    }

    /// The horizontal gap between columns, in logical pixels.
    #[must_use]
    pub const fn column_gap(&self) -> f32 {
        self.column_gap
    }

    /// The vertical gap between rows, in logical pixels.
    #[must_use]
    pub const fn row_gap(&self) -> f32 {
        self.row_gap
    }
}

/// The `(base, fr_weight, floor)` a single track contributes to track sizing.
///
/// * `base` is the track's size before any `fr` space is distributed.
/// * `fr_weight` is its flexible fraction (0 for inflexible tracks).
/// * `floor` is the minimum the track may never drop below after distribution.
///
/// `auto_size` is the content size to use for `Auto` (or an `auto` `minmax`
/// bound): the largest content extent among the items placed in this track.
fn resolve_track(track: &TrackSize, auto_size: f32) -> (f32, f32, f32) {
    match track {
        TrackSize::Fixed(px) => {
            let v = px.max(0.0);
            (v, 0.0, v)
        }
        TrackSize::Auto => {
            let v = auto_size.max(0.0);
            (v, 0.0, v)
        }
        TrackSize::Fr(f) => (0.0, f.max(0.0), 0.0),
        TrackSize::MinMax(min, max) => {
            let floor = resolve_min(min, auto_size);
            match max.as_ref() {
                TrackSize::Fr(f) => (floor, f.max(0.0), floor),
                TrackSize::Fixed(px) => (px.max(0.0).max(floor), 0.0, floor),
                TrackSize::Auto => (auto_size.max(0.0).max(floor), 0.0, floor),
                // A nested `minmax` as the `max` bound is not meaningful; keep the
                // floor so the track never collapses below its `min`.
                TrackSize::MinMax(_, _) => (floor, 0.0, floor),
            }
        }
    }
}

/// Resolves the floor a `minmax` `min` bound imposes. An `fr` min acts as `0`.
fn resolve_min(track: &TrackSize, auto_size: f32) -> f32 {
    match track {
        TrackSize::Fixed(px) => px.max(0.0),
        TrackSize::Auto => auto_size.max(0.0),
        TrackSize::Fr(_) => 0.0,
        TrackSize::MinMax(inner_min, _) => resolve_min(inner_min, auto_size),
    }
}

/// Resolves a list of tracks to concrete pixel sizes along one axis.
///
/// `available` is the content extent along the axis, `gap` the inter-track gap,
/// and `auto_size(i)` supplies the content size for track `i`'s `Auto` (or `auto`
/// `minmax`) resolution. Fixed and auto tracks take their base size; the space
/// left over (after subtracting the gaps and those bases) is shared among the
/// `fr` tracks in proportion to their fractions — the same free-space split
/// [`FlexLayout`](super::FlexLayout) uses for `flex-grow`. Every track is then
/// floored (per its `minmax` `min`) and clamped to be non-negative.
fn resolve_tracks(
    tracks: &[TrackSize],
    available: f32,
    gap: f32,
    auto_size: impl Fn(usize) -> f32,
) -> Vec<f32> {
    let n = tracks.len();
    if n == 0 {
        return Vec::new();
    }
    let gap_span = gap * (n.saturating_sub(1) as f32);
    let free_space = (available - gap_span).max(0.0);

    let mut sizes = Vec::with_capacity(n);
    let mut floors = Vec::with_capacity(n);
    let mut frs = Vec::with_capacity(n);
    for (i, track) in tracks.iter().enumerate() {
        let (base, fr, floor) = resolve_track(track, auto_size(i));
        sizes.push(base);
        frs.push(fr);
        floors.push(floor);
    }

    let sum_base: f32 = sizes.iter().copied().sum();
    let sum_fr: f32 = frs.iter().copied().sum();
    let leftover = (free_space - sum_base).max(0.0);
    if sum_fr > 0.0 && leftover > 0.0 {
        for (size, &fr) in sizes.iter_mut().zip(frs.iter()) {
            *size += leftover * fr / sum_fr;
        }
    }

    for (size, &floor) in sizes.iter_mut().zip(floors.iter()) {
        *size = size.max(floor).max(0.0);
    }
    sizes
}

/// Running start offsets for a track list: `offsets[i] = origin + Σ_{k<i}
/// (sizes[k] + gap)`. Length is `sizes.len()`.
fn track_offsets(sizes: &[f32], gap: f32, origin: f32) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(sizes.len());
    let mut cursor = origin;
    for &size in sizes {
        offsets.push(cursor);
        cursor += size + gap;
    }
    offsets
}

/// The pixel extent a `span`-track cell covers: the sum of the spanned track
/// sizes plus the `span - 1` interior gaps between them.
fn span_extent(sizes: &[f32], start: usize, span: usize, gap: f32) -> f32 {
    let end = (start + span).min(sizes.len());
    let base: f32 = sizes.get(start..end).map_or(0.0, |s| s.iter().copied().sum());
    base + gap * (span.saturating_sub(1) as f32)
}

/// Whether a `rspan × cspan` block anchored at `(row, col)` is entirely free in
/// the occupancy grid. Cells in rows past the grid's current height are free.
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

/// Marks a `rspan × cspan` block occupied, growing `occ` (with `n_cols`-wide
/// rows) to reach the block.
fn mark_block(
    occ: &mut Vec<Vec<bool>>,
    row: usize,
    rspan: usize,
    col: usize,
    cspan: usize,
    n_cols: usize,
) {
    while occ.len() < row + rspan {
        occ.push(vec![false; n_cols]);
    }
    for line in occ.iter_mut().skip(row).take(rspan) {
        for cell in line.iter_mut().skip(col).take(cspan) {
            *cell = true;
        }
    }
}

/// First row (scanning up from 0) where a column-locked block fits.
fn first_row(occ: &[Vec<bool>], col: usize, cspan: usize, rspan: usize, n_cols: usize) -> usize {
    let mut row = 0;
    while !block_free(occ, row, rspan, col, cspan, n_cols) {
        row += 1;
    }
    row
}

/// First column in a fixed row where a row-locked block fits (`0` if the row is
/// already full — a degenerate overlap kept deterministic).
fn first_col(occ: &[Vec<bool>], row: usize, cspan: usize, rspan: usize, n_cols: usize) -> usize {
    if cspan > n_cols {
        return 0;
    }
    (0..=(n_cols - cspan))
        .find(|&col| block_free(occ, row, rspan, col, cspan, n_cols))
        .unwrap_or(0)
}

/// First free `(col, row)` in row-major order for a fully-auto block (first-fit).
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

impl LayoutAlgorithm for GridLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let padding = style.padding;
        let pad_x = padding.left + padding.right;
        let pad_y = padding.top + padding.bottom;

        // Step 1: content area the grid sizes its tracks against. An explicit
        // container `width` / `height` is honoured (resolved against the available
        // max, so a fixed px or a percentage of it both work) so column/row tracks,
        // `fr` distribution, and percentages all resolve against the authored size;
        // an `Auto` dimension falls back to the available content extent, which is
        // byte-identical to the previous fill-available behaviour.
        let content_width = style
            .width
            .resolve(constraints.max.width)
            .unwrap_or(constraints.max.width - pad_x)
            .max(0.0);
        let content_height = style
            .height
            .resolve(constraints.max.height)
            .unwrap_or(constraints.max.height - pad_y)
            .max(0.0);

        // An empty column template still lays out one flexible column so items
        // always have somewhere to go.
        let fallback_cols = [TrackSize::Fr(1.0)];
        let columns: &[TrackSize] = if self.template.columns().is_empty() {
            &fallback_cols
        } else {
            self.template.columns()
        };
        let count = children.len();

        // Resolve each child's line-based placement into a definite/auto anchor
        // plus its column/row span (spec §4.13, a subset). A default placement is
        // all-`Auto` → `(None, 1)` on both axes: pure row-major auto-placement.
        let templ_cols_len = columns.len();
        let templ_rows_len = self.template.rows().len();
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
        let mut n_cols = columns.len().max(1);
        for i in 0..count {
            if let Some(c) = col_anchor[i] {
                n_cols = n_cols.max(c + col_span[i]);
            }
            n_cols = n_cols.max(col_span[i]);
        }

        // Two-phase placement into an occupancy grid. Phase A — items with a
        // definite column and/or row: honour the definite axis, auto-flow any
        // missing axis. Phase B — fully-auto items: row-major first-fit into the
        // first free cell that fits their span. With all-`Auto` placement Phase A
        // is empty and Phase B fills cell `i` in row-major order — byte-identical
        // to the previous `(i % n_cols, i / n_cols)` auto-placement.
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
        for i in 0..count {
            if col_anchor[i].is_some() || row_anchor[i].is_some() {
                continue; // placed in Phase A
            }
            let (c0, r0) = first_fit(&occ, col_span[i], row_span[i], n_cols);
            mark_block(&mut occ, r0, row_span[i], c0, col_span[i], n_cols);
            place_col[i] = c0;
            place_row[i] = r0;
        }

        // Total rows the grid spans (implicit auto rows past the template).
        let n_rows = (0..count)
            .map(|i| place_row[i] + row_span[i])
            .max()
            .unwrap_or(0);

        // Effective column tracks: the template, padded with implicit `Auto`.
        let cols_effective: Vec<TrackSize> = (0..n_cols)
            .map(|c| columns.get(c).cloned().unwrap_or(TrackSize::Auto))
            .collect();

        // Step 2: column track sizing. An `Auto`/`minmax` column sizes to the
        // widest item anchored in it; a spanning item attributes its content
        // width to its FIRST spanned column (a deterministic simplification).
        let loose = BoxConstraints::loose(Size::new(content_width, content_height));
        let mut auto_col = vec![0.0_f32; n_cols];
        for (i, &c0) in place_col.iter().enumerate() {
            if let TrackSize::Auto | TrackSize::MinMax(_, _) = cols_effective[c0] {
                let w = measurer.measure_child(i, loose).width;
                auto_col[c0] = auto_col[c0].max(w);
            }
        }
        let col_sizes =
            resolve_tracks(&cols_effective, content_width, self.column_gap, |c| auto_col[c]);

        // Measure each item with its width fixed to its spanned columns (spanned
        // track widths + interior column gaps), height loose; this feeds both
        // content-sized row heights and the placed item boxes.
        let mut item_size = vec![Size::ZERO; count];
        for i in 0..count {
            let w = span_extent(&col_sizes, place_col[i], col_span[i], self.column_gap);
            item_size[i] = measurer.measure_child(
                i,
                BoxConstraints {
                    min: Size::new(w, 0.0),
                    max: Size::new(w, content_height),
                },
            );
        }

        // Step 3: row track sizing. Explicit rows come from the template; rows
        // past it are implicit (auto). Auto/implicit rows size to the tallest
        // item anchored in them (a spanning item attributes its height to its
        // FIRST spanned row).
        let mut auto_row = vec![0.0_f32; n_rows];
        for i in 0..count {
            let r0 = place_row[i];
            auto_row[r0] = auto_row[r0].max(item_size[i].height);
        }
        let template_rows = self.template.rows();
        let rows_effective: Vec<TrackSize> = (0..n_rows)
            .map(|r| template_rows.get(r).cloned().unwrap_or(TrackSize::Auto))
            .collect();
        let row_sizes =
            resolve_tracks(&rows_effective, content_height, self.row_gap, |r| auto_row[r]);

        // Step 4 & 5: place each item into its (spanned) track cell.
        let col_offsets = track_offsets(&col_sizes, self.column_gap, padding.left);
        let row_offsets = track_offsets(&row_sizes, self.row_gap, padding.top);

        let mut child_boxes = Vec::with_capacity(count);
        // Bounding box of placed children; starts at the padding origin so an
        // empty grid sizes to its padding only.
        let mut max_right = padding.left;
        let mut max_bottom = padding.top;
        for (i, size) in item_size.iter().enumerate() {
            let (c0, cspan) = (place_col[i], col_span[i]);
            let (r0, rspan) = (place_row[i], row_span[i]);
            let x = col_offsets[c0];
            let y = row_offsets[r0];
            let w = span_extent(&col_sizes, c0, cspan, self.column_gap);
            // A single content-sized row keeps the item's own measured height;
            // otherwise the item fills its spanned row tracks (+ interior gaps).
            let h = if rspan == 1 && matches!(rows_effective[r0], TrackSize::Auto) {
                size.height
            } else {
                span_extent(&row_sizes, r0, rspan, self.row_gap)
            };
            child_boxes.push(LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h))));
            max_right = max_right.max(x + w);
            max_bottom = max_bottom.max(y + h);
        }

        // Step 6: border box, clamped. When the container sets an explicit size the
        // border box is that content extent plus padding (so the container *is* the
        // authored size, even if its tracks do not fill it); an `Auto` axis sizes to
        // the children's bounding box plus padding, exactly as before.
        let box_width = if style.width.is_auto() {
            max_right + padding.right
        } else {
            content_width + pad_x
        };
        let box_height = if style.height.is_auto() {
            max_bottom + padding.bottom
        } else {
            content_height + pad_y
        };
        let border_box = Size::new(box_width, box_height);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::{GridLayout, GridTemplate, TrackSize};
    use crate::algorithm::{
        FixedChildMeasurer, GridLine, GridPlacement, LayoutAlgorithm, LayoutChild,
    };
    use crate::model::{Dimension, LayoutStyle};
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
    }

    fn placed(raw: u64, placement: GridPlacement) -> LayoutChild {
        LayoutChild::new(
            NodeId::from_raw(raw),
            LayoutStyle::default().with_grid_placement(placement),
        )
    }

    fn grid(columns: Vec<TrackSize>, rows: Vec<TrackSize>, col_gap: f32, row_gap: f32) -> GridLayout {
        GridLayout::with_template(GridTemplate::new(columns, rows), col_gap, row_gap)
    }

    // (a) `100px 1fr 2fr` across a 400px content width, no gap.
    #[test]
    fn fixed_and_fr_columns_share_leftover() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(10.0, 20.0),
            Size::new(10.0, 20.0),
            Size::new(10.0, 20.0),
        ]);
        let g = grid(
            vec![TrackSize::Fixed(100.0), TrackSize::Fr(1.0), TrackSize::Fr(2.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // 1fr = (400-100)/3 = 100, 2fr = 200.
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size().width, 100.0);
        assert_eq!(b[1].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[1].size().width, 100.0);
        assert_eq!(b[2].origin(), Point::new(200.0, 0.0));
        assert_eq!(b[2].size().width, 200.0);
        assert_eq!(out.size(), Size::new(400.0, 20.0));
    }

    // (b) `1fr 1fr` → two equal 50% columns.
    #[test]
    fn equal_fr_columns_split_evenly() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 150.0);
        assert_eq!(b[1].origin().x, 150.0);
        assert_eq!(b[1].size().width, 150.0);
    }

    // (c) a 10px column gap shifts columns and reduces the fr space.
    #[test]
    fn column_gap_reduces_fr_space() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fixed(100.0), TrackSize::Fr(1.0)],
            vec![],
            10.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // available = 400 - 10 (gap) = 390; fr = 390 - 100 = 290.
        assert_eq!(b[0].size().width, 100.0);
        assert_eq!(b[1].origin().x, 110.0); // 100 + gap 10
        assert_eq!(b[1].size().width, 290.0);
    }

    // (d) auto-placement: 5 items in 2 columns → 3 rows; item 4 at (col0,row2).
    #[test]
    fn row_major_auto_placement() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3), child(4), child(5)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 5]);
        let g = grid(vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // 5 items / 2 cols = 3 rows (rows tall 10 each).
        assert_eq!(b[3].origin(), Point::new(100.0, 10.0)); // col1,row1
        assert_eq!(b[4].origin(), Point::new(0.0, 20.0)); // col0,row2
        assert_eq!(out.size(), Size::new(200.0, 30.0));
    }

    // (e) implicit auto rows are sized to the tallest item in the row.
    #[test]
    fn implicit_rows_size_to_tallest_item() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3), child(4)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(0.0, 20.0),
            Size::new(0.0, 35.0), // tallest in row 0
            Size::new(0.0, 15.0),
            Size::new(0.0, 5.0),
        ]);
        let g = grid(vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Row 0 tallest = 35 → row 1 starts at y = 35.
        assert_eq!(b[2].origin(), Point::new(0.0, 35.0));
        assert_eq!(b[3].origin(), Point::new(100.0, 35.0));
        // Border box height = row0(35) + row1(15) = 50.
        assert_eq!(out.size(), Size::new(200.0, 50.0));
    }

    // (f) an `Auto` column sizes to its items' content width.
    #[test]
    fn auto_column_sizes_to_content() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(80.0, 20.0), // col 0 (auto) content
            Size::new(30.0, 20.0), // col 1 (fr)
        ]);
        let g = grid(vec![TrackSize::Auto, TrackSize::Fr(1.0)], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Auto column = 80; fr column = 300 - 80 = 220.
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 80.0);
        assert_eq!(b[1].origin().x, 80.0);
        assert_eq!(b[1].size().width, 220.0);
    }

    // (g) `minmax(100px, 1fr)` never resolves below 100.
    #[test]
    fn minmax_floors_fr_column() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        // Two minmax(100px,1fr) columns in a 150px width: 2*100 = 200 > 150, so
        // each is floored at 100 (no room for fr growth).
        let g = grid(
            vec![
                TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0)),
                TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0)),
            ],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(150.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 100.0);
        assert_eq!(b[1].size().width, 100.0);
        assert_eq!(b[1].origin().x, 100.0);
    }

    // (h) regression: `GridLayout::new(1)` lays a single equal column.
    #[test]
    fn new_one_is_single_equal_column() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 20.0), Size::new(50.0, 30.0)]);
        let out = GridLayout::new(1).layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(120.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // One column filling the full content width; items stack vertically.
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size().width, 120.0);
        assert_eq!(b[1].origin(), Point::new(0.0, 20.0));
        assert_eq!(GridLayout::new(1).columns(), 1);
        assert_eq!(GridLayout::new(0).columns(), 1);
    }

    // ---- Regressions carried over from the uniform-grid implementation. ----

    #[test]
    fn three_children_fill_two_columns_over_two_rows() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(100.0, 20.0),
            Size::new(100.0, 30.0),
            Size::new(100.0, 10.0),
        ]);
        let out = GridLayout::new(2).layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size(), Size::new(100.0, 20.0));
        assert_eq!(b[1].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[1].size(), Size::new(100.0, 30.0));
        assert_eq!(b[2].origin(), Point::new(0.0, 30.0));
        assert_eq!(b[2].size(), Size::new(100.0, 10.0));
        assert_eq!(out.size(), Size::new(200.0, 40.0));
    }

    #[test]
    fn gaps_offset_columns_and_rows() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(0.0, 20.0),
            Size::new(0.0, 20.0),
            Size::new(0.0, 10.0),
        ]);
        let g = GridLayout::new(2).with_column_gap(10.0).with_row_gap(5.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(210.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[1].origin(), Point::new(110.0, 0.0));
        assert_eq!(b[2].origin(), Point::new(0.0, 25.0));
        assert_eq!(b[0].size(), Size::new(100.0, 20.0));
    }

    #[test]
    fn zero_columns_clamps_to_one() {
        let g = GridLayout::new(0);
        assert_eq!(g.columns(), 1);
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 20.0), Size::new(50.0, 30.0)]);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(50.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(0.0, 20.0));
    }

    #[test]
    fn padding_offsets_grid_and_grows_box() {
        let style = LayoutStyle {
            padding: Insets::uniform(5.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(90.0, 40.0)]);
        let out = GridLayout::new(1).layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(5.0, 5.0));
        assert_eq!(out.size(), Size::new(100.0, 50.0));
    }

    // Explicit container width bounds the fr tracks: `width:300px` with `1fr 1fr`
    // under a 500px available extent → two 150px columns, not 250.
    #[test]
    fn explicit_width_bounds_fr_columns() {
        let style = LayoutStyle {
            width: Dimension::Px(300.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // fr tracks resolve against the explicit 300, not the 500 available.
        assert_eq!(b[0].size().width, 150.0);
        assert_eq!(b[1].origin().x, 150.0);
        assert_eq!(b[1].size().width, 150.0);
        // The container is its authored 300px wide.
        assert_eq!(out.size().width, 300.0);
    }

    // Explicit container height bounds the fr rows: `height:200px` with two `1fr`
    // rows → each row 100 tall, and the container is 200 tall.
    #[test]
    fn explicit_height_bounds_fr_rows() {
        let style = LayoutStyle {
            height: Dimension::Px(200.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fr(1.0)],
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 500.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Rows share the explicit 200 → 100 each; second child in the second row.
        assert_eq!(b[1].origin().y, 100.0);
        assert_eq!(out.size().height, 200.0);
    }

    // Regression: with no explicit width, tracks still fill the available extent.
    #[test]
    fn auto_width_fills_available_columns() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Auto width → fr tracks fill the 500 available → two 250px columns.
        assert_eq!(b[0].size().width, 250.0);
        assert_eq!(b[1].size().width, 250.0);
        assert_eq!(out.size().width, 500.0);
    }

    // ---- Line-based placement + spanning. ----

    fn three_col_100() -> GridLayout {
        grid(
            vec![
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(100.0),
            ],
            vec![],
            0.0,
            0.0,
        )
    }

    // (a) `grid-column: 1 / 3` spans cols 0-1: width 200 at x=0.
    #[test]
    fn column_line_to_line_spans_two_tracks() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Line(1), GridLine::Line(3)),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0)]);
        let out = three_col_100().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size().width, 200.0);
    }

    // (b) `grid-column: span 2` auto-placed occupies cols 0-1 (width 200 at x=0).
    #[test]
    fn column_span_auto_placed_occupies_two_cells() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Auto, GridLine::Span(2)),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0)]);
        let out = three_col_100().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size().width, 200.0);
    }

    // (c) an explicit item at column line 2 + an auto item: the auto item flows
    // around the occupied cell, landing in column 0.
    #[test]
    fn auto_item_flows_around_explicit_cell() {
        let style = LayoutStyle::default();
        let children = [
            placed(1, GridPlacement::column(GridLine::Line(2), GridLine::Auto)),
            child(2),
        ];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 20.0), Size::new(0.0, 20.0)]);
        let out = three_col_100().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Explicit item on column 1 (x=100), auto item flows to column 0 (x=0).
        assert_eq!(b[0].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(0.0, 0.0));
    }

    // (d) `grid-row: 2` puts an item on row index 1. Row 0 is a fixed 50px track
    // so the row boundary is a concrete coordinate.
    #[test]
    fn explicit_row_places_on_second_row() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::row(GridLine::Line(2), GridLine::Auto),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 30.0)]);
        let g = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::Fixed(50.0)], // row 0 fixed; row 1 implicit auto
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Row 0 (50px) is empty; the item sits on row 1 at y = 50.
        assert_eq!(b[0].origin(), Point::new(0.0, 50.0));
    }

    // (e) a span that does not fit the remaining row wraps to the next row.
    #[test]
    fn span_that_does_not_fit_wraps_to_next_row() {
        let style = LayoutStyle::default();
        let children = [
            child(1), // auto → (col0,row0)
            placed(2, GridPlacement::column(GridLine::Auto, GridLine::Span(2))),
        ];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 15.0)]);
        let g = grid(
            vec![TrackSize::Fixed(100.0), TrackSize::Fixed(100.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // child0 fills (0,0); the span-2 item cannot fit beside it, so it wraps
        // to row 1 (y = row0 height = 10), spanning the full 200px width.
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(0.0, 10.0));
        assert_eq!(b[1].size().width, 200.0);
    }

    // (f) regression: all-`Auto` placement reproduces the row-major result.
    #[test]
    fn all_auto_placement_is_row_major() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3), child(4), child(5)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 5]);
        let g = grid(vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[3].origin(), Point::new(100.0, 10.0)); // col1,row1
        assert_eq!(b[4].origin(), Point::new(0.0, 20.0)); // col0,row2
        assert_eq!(out.size(), Size::new(200.0, 30.0));
    }

    // (g) a column gap is applied across a span's interior.
    #[test]
    fn column_gap_applies_across_span_interior() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Line(1), GridLine::Line(3)),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0)]);
        let g = grid(
            vec![
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(100.0),
            ],
            vec![],
            10.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(320.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Spans cols 0-1: 100 + 100 + one interior 10px gap = 210, at x=0.
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 210.0);
    }

    #[test]
    fn empty_grid_sizes_to_padding() {
        let style = LayoutStyle {
            padding: Insets::uniform(4.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::default();
        let out = GridLayout::new(3).layout(
            &style,
            &[],
            BoxConstraints::loose(Size::new(60.0, 60.0)),
            &mut measurer,
        );
        assert!(out.is_empty());
        assert_eq!(out.size(), Size::new(8.0, 8.0));
    }
}
