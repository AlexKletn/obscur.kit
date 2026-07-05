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
/// 1. **Content area** — the container `padding` is subtracted from
///    `constraints.max` to get the content width/height (each clamped ≥ 0).
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
/// 4. **Auto-placement (row-major)** — with `n_cols = columns.len().max(1)`,
///    item `i` occupies column `i % n_cols` of row `i / n_cols`; the grid has
///    `ceil(n_items / n_cols)` rows.
/// 5. **Placement + sizing** — each item is measured with its width fixed to its
///    column track; for content-sized (`Auto` / implicit) rows its height is left
///    loose (its own measured height), otherwise its height is fixed to the row
///    track. It is placed at
///    `(padding.left + Σ(prior column widths) + col * column_gap,
///      padding.top  + Σ(prior row heights)  + row * row_gap)`.
/// 6. **Border box** — the bounding box of the placed children plus the container
///    `padding`, then clamped into `constraints`.
///
/// # Out of scope (later steps)
///
/// This is real *track sizing* but still a subset: no explicit line-based
/// placement, no row/column spanning, no dense auto-placement packing, no named
/// lines, and no alignment of an item within its track. Those belong to later
/// work; the CSS parsing and runtime wiring of templates is step 2.
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

        // Step 1: content area available to the grid (clamped non-negative).
        let content_width = (constraints.max.width - pad_x).max(0.0);
        let content_height = (constraints.max.height - pad_y).max(0.0);

        // An empty column template still lays out one flexible column so items
        // always have somewhere to go.
        let fallback_cols = [TrackSize::Fr(1.0)];
        let columns: &[TrackSize] = if self.template.columns().is_empty() {
            &fallback_cols
        } else {
            self.template.columns()
        };
        let n_cols = columns.len().max(1);
        let count = children.len();

        // Row-major auto-placement: item i → (col i % n_cols, row i / n_cols).
        let n_rows = count.div_ceil(n_cols);

        // Step 2: column track sizing. `Auto` columns size to the widest item
        // placed in them, measured under loose constraints.
        let loose = BoxConstraints::loose(Size::new(content_width, content_height));
        let mut auto_col = vec![0.0_f32; n_cols];
        for (i, _child) in children.iter().enumerate() {
            if let TrackSize::Auto | TrackSize::MinMax(_, _) = columns[i % n_cols] {
                let w = measurer.measure_child(i, loose).width;
                let slot = &mut auto_col[i % n_cols];
                *slot = slot.max(w);
            }
        }
        let col_sizes = resolve_tracks(columns, content_width, self.column_gap, |c| auto_col[c]);

        // Measure each item with its width fixed to its column, height loose; this
        // feeds both content-sized row heights and the placed item boxes.
        let mut item_size = vec![Size::ZERO; count];
        for (i, size) in item_size.iter_mut().enumerate() {
            let w = col_sizes[i % n_cols];
            *size = measurer.measure_child(
                i,
                BoxConstraints {
                    min: Size::new(w, 0.0),
                    max: Size::new(w, content_height),
                },
            );
        }

        // Step 3: row track sizing. Explicit rows come from the template; rows
        // past it are implicit (auto). Auto/implicit rows size to the tallest
        // item in the row.
        let mut auto_row = vec![0.0_f32; n_rows];
        for (i, size) in item_size.iter().enumerate() {
            let row = i / n_cols;
            auto_row[row] = auto_row[row].max(size.height);
        }
        let template_rows = self.template.rows();
        let rows_effective: Vec<TrackSize> = (0..n_rows)
            .map(|r| {
                template_rows
                    .get(r)
                    .cloned()
                    .unwrap_or(TrackSize::Auto)
            })
            .collect();
        let row_sizes =
            resolve_tracks(&rows_effective, content_height, self.row_gap, |r| auto_row[r]);

        // Step 4 & 5: place each item into its track cell.
        let col_offsets = track_offsets(&col_sizes, self.column_gap, padding.left);
        let row_offsets = track_offsets(&row_sizes, self.row_gap, padding.top);

        let mut child_boxes = Vec::with_capacity(count);
        // Bounding box of placed children; starts at the padding origin so an
        // empty grid sizes to its padding only.
        let mut max_right = padding.left;
        let mut max_bottom = padding.top;
        for (i, size) in item_size.iter().enumerate() {
            let col = i % n_cols;
            let row = i / n_cols;
            let x = col_offsets[col];
            let y = row_offsets[row];
            let w = col_sizes[col];
            // Content-sized rows keep the item's own measured height; sized rows
            // fix the item to the track height.
            let h = if matches!(rows_effective[row], TrackSize::Auto) {
                size.height
            } else {
                row_sizes[row]
            };
            child_boxes.push(LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h))));
            max_right = max_right.max(x + w);
            max_bottom = max_bottom.max(y + h);
        }

        // Step 6: border box = children bounding box + padding, clamped.
        let border_box = Size::new(max_right + padding.right, max_bottom + padding.bottom);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::{GridLayout, GridTemplate, TrackSize};
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::LayoutStyle;
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
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
