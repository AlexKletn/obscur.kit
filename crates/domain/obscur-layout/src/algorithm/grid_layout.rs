#![doc = include_str!("grid_layout.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{JustifyContent, LayoutBox, LayoutStyle};

use super::grid::{GridTemplate, TrackSize};
use super::grid_item_alignment::{CellAlign, align_offset, item_axis_align};
use super::grid_item_placement::PlacedItems;
use super::grid_track_positions::{collapsed_tracks, distributed_track_offsets, span_extent};
use super::grid_track_sizing::{
    Contributions, KINDS, Kind, implicit_track, resolve_tracks, track_takes,
};
use super::{
    ChildMeasurer, ChildPercentBasis, LayoutAlgorithm, LayoutChild, LayoutOutput, clamp_content,
};

#[doc = include_str!("grid_layout.doc/GridLayout.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridLayout {
    #[doc = include_str!("grid_layout.doc/template.md")]
    template: GridTemplate,
    #[doc = include_str!("grid_layout.doc/column_gap.md")]
    column_gap: f32,
    #[doc = include_str!("grid_layout.doc/row_gap.md")]
    row_gap: f32,
}

impl GridLayout {
    #[doc = include_str!("grid_layout.doc/new.md")]
    #[must_use]
    pub fn new(columns: usize) -> Self {
        let n = columns.max(1);
        Self {
            template: GridTemplate::columns_only(vec![TrackSize::Fr(1.0); n]),
            column_gap: 0.0,
            row_gap: 0.0,
        }
    }

    #[doc = include_str!("grid_layout.doc/with_template.md")]
    #[must_use]
    pub fn with_template(template: GridTemplate, column_gap: f32, row_gap: f32) -> Self {
        Self {
            template,
            column_gap,
            row_gap,
        }
    }

    #[doc = include_str!("grid_layout.doc/with_column_gap.md")]
    #[must_use]
    pub fn with_column_gap(mut self, column_gap: f32) -> Self {
        self.column_gap = column_gap;
        self
    }

    #[doc = include_str!("grid_layout.doc/with_row_gap.md")]
    #[must_use]
    pub fn with_row_gap(mut self, row_gap: f32) -> Self {
        self.row_gap = row_gap;
        self
    }

    #[doc = include_str!("grid_layout.doc/template_2.md")]
    #[must_use]
    pub const fn template(&self) -> &GridTemplate {
        &self.template
    }

    #[doc = include_str!("grid_layout.doc/columns.md")]
    #[must_use]
    pub fn columns(&self) -> usize {
        self.template.columns().explicit_len().max(1)
    }

    #[doc = include_str!("grid_layout.doc/column_gap_2.md")]
    #[must_use]
    pub const fn column_gap(&self) -> f32 {
        self.column_gap
    }

    #[doc = include_str!("grid_layout.doc/row_gap_2.md")]
    #[must_use]
    pub const fn row_gap(&self) -> f32 {
        self.row_gap
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
        // `box-sizing` governs how an explicit container size maps to the content
        // extent the tracks size against: `content-box` (the default) takes it
        // as-is, `border-box` subtracts the axis padding (floored at `0`). The
        // `Auto` branch already subtracts padding, so both modes are consistent
        // with the flex container — an explicitly-sized grid stays at its authored
        // (border-box) size with no phantom inflation.
        let content_width = style
            .width
            .resolve(constraints.max.width)
            .map_or(constraints.max.width - pad_x, |w| {
                style.box_sizing.content_extent(w, pad_x)
            })
            .max(0.0);
        let content_height = style
            .height
            .resolve(constraints.max.height)
            .map_or(constraints.max.height - pad_y, |h| {
                style.box_sizing.content_extent(h, pad_y)
            })
            .max(0.0);

        // Expand each axis's track list into a concrete track vector against the
        // content extent: a `repeat(auto-fill | auto-fit, …)`'s repetition count
        // depends on how many copies fit that extent (spec §4.1). For a template
        // WITHOUT an auto-repeat this is byte-identical to its explicit track
        // vector, so every non-auto-repeat template lays out exactly as before.
        // `col_autofit` / `row_autofit` carry the generated `auto-fit` track range
        // (if any) so empty ones collapse after placement.
        let (columns_expanded, col_autofit) = self
            .template
            .columns()
            .expand(content_width, self.column_gap);
        let (rows_expanded, row_autofit) =
            self.template.rows().expand(content_height, self.row_gap);

        // An empty column template still lays out one flexible column so items
        // always have somewhere to go.
        let fallback_cols = [TrackSize::Fr(1.0)];
        let columns: &[TrackSize] = if columns_expanded.is_empty() {
            &fallback_cols
        } else {
            &columns_expanded
        };
        let count = children.len();

        // Two-phase placement into an occupancy grid: every item's cell and
        // span, plus the column and row counts the grid needs once implicit
        // tracks are added. `grid-auto-flow` decides the iteration axis and
        // whether the placement cursor backfills earlier holes; the whole of
        // that machinery is [`PlacedItems`].
        let PlacedItems {
            col: place_col,
            row: place_row,
            col_span,
            row_span,
            n_cols,
            n_rows,
        } = PlacedItems::resolve(
            children,
            style.grid_auto_flow,
            columns.len(),
            rows_expanded.len(),
        );

        // Effective column tracks: the explicit template, padded with implicit
        // tracks sized by `grid-auto-columns` (cycled; empty → `Auto`).
        let auto_cols = self.template.auto_columns();
        let explicit_cols = columns.len();
        let cols_effective: Vec<TrackSize> = (0..n_cols)
            .map(|c| {
                columns
                    .get(c)
                    .cloned()
                    .unwrap_or_else(|| implicit_track(auto_cols, c - explicit_cols))
            })
            .collect();

        // Step 2: column track sizing via the canonical §11 pipeline. Each item
        // crossing an intrinsic (`auto`/`fr`/intrinsic-`minmax`) column supplies
        // a content-width contribution measured under a loose constraint; the
        // algorithm distributes each spanning item's contribution across ALL its
        // spanned tracks (not just the first). `justify-content: stretch`/`normal`
        // (the default) routes leftover space to §11.8 stretch of `auto` columns.
        let loose = BoxConstraints::loose(Size::new(content_width, content_height));
        let col_definite = !style.width.is_auto();
        let col_stretch = matches!(
            style.justify_content,
            JustifyContent::FlexStart | JustifyContent::Stretch
        );
        let mut col_items: Vec<(usize, usize, Contributions)> = Vec::with_capacity(count);
        for i in 0..count {
            let (start, span) = (place_col[i], col_span[i]);
            let spanned = &cols_effective[start..(start + span).min(cols_effective.len())];
            // One measure per intrinsic kind this item's span actually crosses.
            // The `min-content` / `max-content` probes offer no space / unbounded
            // space along the INLINE axis, exactly as `intrinsic_inline_size` does
            // for a flow box: the item answers with the width it needs rather than
            // the width it was given (`BoxConstraints::constrain_allowing_overflow`).
            let mut contributions = Contributions::default();
            for kind in KINDS {
                if !spanned.iter().any(|t| track_takes(t, kind)) {
                    continue;
                }
                let offer = match kind {
                    Kind::Auto => loose,
                    Kind::Min => BoxConstraints::loose(Size::new(0.0, content_height)),
                    Kind::Max => BoxConstraints::loose(Size::new(f32::INFINITY, content_height)),
                };
                *contributions.slot(kind) = measurer.measure_child(i, offer).width;
            }
            col_items.push((start, span, contributions));
        }
        let mut col_sizes = resolve_tracks(
            &cols_effective,
            content_width,
            self.column_gap,
            col_definite,
            col_stretch,
            &col_items,
        );
        // `auto-fit`: a generated column no item landed in collapses to zero width
        // (its surrounding gaps collapse in `track_offsets`).
        let col_collapsed = collapsed_tracks(n_cols, col_autofit, &place_col, &col_span);
        for (size, &collapsed) in col_sizes.iter_mut().zip(col_collapsed.iter()) {
            if collapsed {
                *size = 0.0;
            }
        }

        // How each item is aligned inside its own grid area, on both axes: a
        // per-item `justify-self` / `align-self` (when not `auto`) overrides the
        // container's `justify-items` / `align-items` default, and `stretch` is
        // then tested against the item's computed size (see `item_axis_align`).
        // Resolved once, here, because two passes need the same answer: the
        // measure below has to offer an item the inline size it will actually be
        // laid out at, and the arrangement further down has to place it there.
        // When those two disagreed, an item whose size kept it out of the stretch
        // was measured at the full track anyway, so a content-sized row was
        // sized from a reflow that never happens.
        let item_align: Vec<(CellAlign, CellAlign)> = children
            .iter()
            .map(|c| {
                let cs = c.style();
                (
                    item_axis_align(cs.justify_self.resolve(style.justify_items), cs.width),
                    item_axis_align(cs.align_self.resolve(style.align_items), cs.height),
                )
            })
            .collect();

        // Measure each item with its width fixed to its spanned columns (spanned
        // track widths + interior column gaps), height loose; this feeds both
        // content-sized row heights and the placed item boxes. "Fixed" only for
        // an item the inline axis really does stretch: one that falls back to
        // flex-start keeps its own width, so it is offered the area as an upper
        // bound and left to shrink to fit.
        //
        // The basis is stated per axis, and the axes differ because the phase does:
        //
        // * **inline** — `w`, the item's grid area along that axis. Identical to
        //   the `max.width` the fallback would have used, said out loud so the
        //   measure and the arrange agree by construction rather than by accident.
        // * **block** — [`f32::INFINITY`], because the rows are sized in step 3
        //   *from these very measurements*: at this moment the item's block axis is
        //   genuinely indefinite, and there is no area size to state. The fallback
        //   to `max.height` used the grid container's content height, which made a
        //   `height: 50%` item contribute half the container to the row that was
        //   being sized by it — a number the spec never asks for. Against an
        //   indefinite size a percentage **behaves as auto**
        //   (`specs/w3c/css-sizing-3.txt:554`), so the item contributes its content
        //   height, the row is sized from that, and the arranging pass below then
        //   states the resolved area (`child_bases`) — which is the two-pass shape
        //   css-grid-2 §12 describes, with no cycle in it.
        let mut item_size = vec![Size::ZERO; count];
        for i in 0..count {
            let w = span_extent(&col_sizes, place_col[i], col_span[i], self.column_gap);
            let min_w = if item_align[i].0 == CellAlign::Stretch {
                w
            } else {
                0.0
            };
            item_size[i] = measurer.measure_child(
                i,
                BoxConstraints {
                    min: Size::new(min_w, 0.0),
                    max: Size::new(w, content_height),
                    percent_basis: Some(Size::new(w, f32::INFINITY)),
                },
            );
        }

        // Step 3: row track sizing via the same §11 pipeline. Explicit rows come
        // from the template; rows past it are implicit, sized by `grid-auto-rows`
        // (cycled; empty → auto). Each item crossing an intrinsic row contributes
        // its measured height (with its width already fixed to its column span);
        // the algorithm distributes a row-spanning item's height across its
        // spanned rows. `align-content: stretch`/`normal` routes to §11.8.
        let auto_rows = self.template.auto_rows();
        let explicit_rows = rows_expanded.len();
        let rows_effective: Vec<TrackSize> = (0..n_rows)
            .map(|r| {
                rows_expanded
                    .get(r)
                    .cloned()
                    .unwrap_or_else(|| implicit_track(auto_rows, r - explicit_rows))
            })
            .collect();
        let row_definite = !style.height.is_auto();
        let row_stretch = matches!(
            style.align_content,
            JustifyContent::FlexStart | JustifyContent::Stretch
        );
        let mut row_items: Vec<(usize, usize, Contributions)> = Vec::with_capacity(count);
        for i in 0..count {
            let (start, span) = (place_row[i], row_span[i]);
            let spanned = &rows_effective[start..(start + span).min(rows_effective.len())];
            // In the BLOCK axis all three contributions are the same number, and it
            // needs no probe: the item's width is already fixed to its column span,
            // and at a fixed width a box does not reflow vertically — its
            // min-content, max-content and natural heights coincide. So a
            // `min-content` row is fed the height already measured above, and costs
            // no extra measure at all (the inline axis is where they diverge).
            let mut contributions = Contributions::default();
            for kind in KINDS {
                if spanned.iter().any(|t| track_takes(t, kind)) {
                    *contributions.slot(kind) = item_size[i].height;
                }
            }
            row_items.push((start, span, contributions));
        }
        let mut row_sizes = resolve_tracks(
            &rows_effective,
            content_height,
            self.row_gap,
            row_definite,
            row_stretch,
            &row_items,
        );
        // `auto-fit` on the row axis: collapse the generated rows with no item.
        let row_collapsed = collapsed_tracks(n_rows, row_autofit, &place_row, &row_span);
        for (size, &collapsed) in row_sizes.iter_mut().zip(row_collapsed.iter()) {
            if collapsed {
                *size = 0.0;
            }
        }

        // Step 4 & 5: place each item into its (spanned) track cell. Leftover free
        // space on each axis (when the tracks do not fill the content extent) is
        // distributed per `justify-content` (columns / inline axis) and
        // `align-content` (rows / block axis); the default `FlexStart` on both
        // packs the tracks at the start — byte-identical to before.
        let col_offsets = distributed_track_offsets(
            &col_sizes,
            self.column_gap,
            padding.left,
            &col_collapsed,
            content_width,
            style.justify_content,
        );
        let row_offsets = distributed_track_offsets(
            &row_sizes,
            self.row_gap,
            padding.top,
            &row_collapsed,
            content_height,
            style.align_content,
        );

        let mut child_boxes = Vec::with_capacity(count);
        // The containing block each item's percentages resolve against — its GRID
        // AREA, not this container's content box (`specs/w3c/css-grid-2.txt:712`:
        // "A grid item's grid area forms the containing block into which it is laid
        // out"; §6.2 at `:1042`: "A grid item is sized within the containing block
        // defined by its grid area"). Filled in alongside `child_boxes` below,
        // where the area is known; see the note there for why only the inline axis
        // is stated.
        let mut child_bases = Vec::with_capacity(count);
        // Bounding box of placed children; starts at the padding origin so an
        // empty grid sizes to its padding only.
        let mut max_right = padding.left;
        let mut max_bottom = padding.top;
        for (i, size) in item_size.iter().enumerate() {
            let (c0, cspan) = (place_col[i], col_span[i]);
            let (r0, rspan) = (place_row[i], row_span[i]);
            let cell_x = col_offsets[c0];
            let cell_y = row_offsets[r0];
            let cell_w = span_extent(&col_sizes, c0, cspan, self.column_gap);
            // A single content-sized row keeps the item's own measured height;
            // otherwise the cell fills its spanned row tracks (+ interior gaps).
            let cell_h = if rspan == 1 && matches!(rows_effective[r0], TrackSize::Auto) {
                size.height
            } else {
                span_extent(&row_sizes, r0, rspan, self.row_gap)
            };

            // This item's effective alignment on each axis, resolved above: the
            // inline axis is the column (x) axis, the block axis is the row (y)
            // axis. An axis that is not `Stretch` — either because the author
            // aligned it, or because the item has a computed size of its own and
            // `stretch` therefore fell back to flex-start — goes down the
            // measure-and-place path below.
            let (inline, block) = item_align[i];

            let (x, y, w, h) = if inline == CellAlign::Stretch && block == CellAlign::Stretch {
                // Default: the item fills its cell — byte-identical to before.
                (cell_x, cell_y, cell_w, cell_h)
            } else {
                // At least one axis aligns the item within its cell. Re-measure
                // the natural size: an axis that still stretches keeps a size
                // fixed to the cell extent, an aligned axis is loose up to it.
                //
                // `percent_basis` stays unstated here for the same reason as the
                // measure above: `max` is `(cell_w, cell_h)` — the grid area on
                // both axes — so a percentage size already resolves against the
                // area through the fallback, and stating it would only add a
                // distinct key to the caller's measure memoization for an
                // identical answer.
                let natural = measurer.measure_child(
                    i,
                    BoxConstraints {
                        min: Size::new(
                            if inline == CellAlign::Stretch {
                                cell_w
                            } else {
                                0.0
                            },
                            if block == CellAlign::Stretch {
                                cell_h
                            } else {
                                0.0
                            },
                        ),
                        max: Size::new(cell_w, cell_h),
                        percent_basis: None,
                    },
                );
                let iw = if inline == CellAlign::Stretch {
                    cell_w
                } else {
                    natural.width
                };
                let ih = if block == CellAlign::Stretch {
                    cell_h
                } else {
                    natural.height
                };
                (
                    cell_x + align_offset(inline, cell_w, iw),
                    cell_y + align_offset(block, cell_h, ih),
                    iw,
                    ih,
                )
            };
            child_boxes.push(LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h))));
            // This item's containing block, for its own percentages and therefore
            // for everything its subtree inherits. `cell_w` is the item's grid area
            // along the inline axis — and it is *precisely* the number the item's
            // own used inline size was resolved against a few lines above, where
            // every item is measured with `min.width == max.width == span_extent`.
            // Stating it here is what makes the caller's second, arranging pass
            // agree with that measure: without it the caller falls back to this
            // grid container's content box, so an item at `width: 100%` came out
            // 211 (its track) while its children resolved `50%` against 432 (the
            // container) — a containing block the item itself never had
            // (`docs/external-consumer-audit.md`, 2-4 (a)).
            //
            // The BLOCK axis is `cell_h`, for the same reason and by the same rule.
            // An earlier round left it unstated on the worry that for a
            // content-sized row `cell_h` *is* the item's own measured height, and a
            // box cannot be its own containing block. `specs/w3c/css-grid-2.txt:3527`
            // answers that worry by phase order rather than by exception: "Lay out
            // the grid items into their respective containing blocks. Each grid
            // area's width and height are considered definite for this purpose."
            // Track sizing has already run by the time this loop places anything —
            // `item_size[i]` was measured against the container, the rows were then
            // resolved from those measurements, and only now are items laid into
            // their areas. There is no cycle left to close: the number below is a
            // resolved output of a finished phase, not an input to itself.
            //
            // Stating the number the item was ACTUALLY GIVEN is the whole point.
            // `cell_w`/`cell_h` are exactly the extents this loop lays the item
            // into, so the item's used size and the basis its subtree inherits
            // cannot disagree — and their disagreeing is precisely what defect 2-4(a)
            // was (`docs/external-consumer-audit.md`). Handing down the container's
            // content box instead is how a `width: 100%` item came out 211 (its
            // track) while its children resolved `50%` against 432 (the container),
            // a containing block the item itself never had; leaving the block axis
            // to that same fallback left the identical defect standing in the other
            // axis.
            child_bases.push(ChildPercentBasis::with_size(cell_w, cell_h));
            // The grid's own extent is defined by its tracks (the cell), not by
            // where an aligned item sits inside it, so the bounding box grows by
            // the cell — byte-identical to the fill case.
            max_right = max_right.max(cell_x + cell_w);
            max_bottom = max_bottom.max(cell_y + cell_h);
        }

        // Step 6: border box, clamped. When the container sets an explicit size the
        // border box is that content extent plus padding (so the container *is* the
        // authored size, even if its tracks do not fill it); an `Auto` axis sizes to
        // the children's bounding box plus padding, exactly as before.
        //
        // `max_right`/`max_bottom` are accumulated from the content origin (they
        // start at `padding.left`/`padding.top`), so subtracting that origin back
        // off yields the CONTENT extent — the axis `clamp_content` operates on.
        let own_content_width = if style.width.is_auto() {
            max_right - padding.left
        } else {
            content_width
        };
        let own_content_height = if style.height.is_auto() {
            max_bottom - padding.top
        } else {
            content_height
        };

        // Clamp the container's OWN box into its authored min/max, exactly as flow
        // and flex do (spec §4.2). Without this an authored `min-height` /
        // `max-width` on a `display: grid` element is silently ignored — the same
        // defect flow and flex carried before `clamp_content` was factored out.
        let own_w = clamp_content(
            own_content_width,
            style.min_width,
            style.max_width,
            constraints.max.width,
            pad_x,
            style.box_sizing,
        );
        let own_h = clamp_content(
            own_content_height,
            style.min_height,
            style.max_height,
            constraints.max.height,
            pad_y,
            style.box_sizing,
        );
        let border_box = Size::new(own_w + pad_x, own_h + pad_y);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes).with_child_percent_bases(child_bases)
    }
}

#[cfg(test)]
mod tests {
    use super::{GridLayout, GridTemplate, TrackSize};
    use crate::algorithm::{
        AutoRepeat, AutoRepeatMode, ChildMeasurer, ChildPercentBasis, FixedChildMeasurer,
        FlowLayout, GridAutoFlow, GridLine, GridPlacement, LayoutAlgorithm, LayoutChild, TrackList,
    };
    use crate::model::{BoxSizing, Dimension, JustifyContent, LayoutStyle};
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

    fn grid(
        columns: Vec<TrackSize>,
        rows: Vec<TrackSize>,
        col_gap: f32,
        row_gap: f32,
    ) -> GridLayout {
        GridLayout::with_template(GridTemplate::new(columns, rows), col_gap, row_gap)
    }

    // ---- §4.2 own-box min/max clamp (parity with flow + flex) ---------------

    #[test]
    fn min_height_floors_a_short_grid() {
        // One 20-tall row, but `min-height: 200` floors the container. Grid used
        // to honour only the parent constraints, so an authored `min-height` on a
        // `display: grid` element was silently dropped — flow and flex clamped.
        let style = LayoutStyle {
            min_height: Dimension::Px(200.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 20.0)]);
        let out = grid(vec![TrackSize::Fixed(100.0)], vec![], 0.0, 0.0).layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().height, 200.0);
    }

    #[test]
    fn max_height_caps_a_tall_grid() {
        let style = LayoutStyle {
            max_height: Dimension::Px(50.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 300.0)]);
        let out = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::Fixed(300.0)],
            0.0,
            0.0,
        )
        .layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().height, 50.0);
    }

    #[test]
    fn max_width_caps_an_explicitly_sized_grid() {
        // The clamp applies to an authored width too, not just the `Auto` axis.
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            max_width: Dimension::Px(120.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 20.0)]);
        let out = grid(vec![TrackSize::Fixed(100.0)], vec![], 0.0, 0.0).layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().width, 120.0);
    }

    #[test]
    fn the_clamp_is_measured_from_the_content_box_under_padding() {
        // The clamp operates on the CONTENT extent, then padding is added back.
        // `min-height: 200` under `content-box` sizing means 200 of *content*, so
        // the border box is 200 + 10 + 10 — the same rule flow applies. This is
        // what makes the `max_bottom - padding.top` round-trip load-bearing.
        let style = LayoutStyle {
            min_height: Dimension::Px(200.0),
            padding: Insets::uniform(10.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 20.0)]);
        let out = grid(vec![TrackSize::Fixed(100.0)], vec![], 0.0, 0.0).layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().height, 220.0);
    }

    #[test]
    fn auto_min_max_leaves_the_grid_byte_identical() {
        // The clamp must be a no-op when nothing is authored: an `Auto`-height
        // grid still sizes to its tracks' bounding box plus padding.
        let style = LayoutStyle {
            padding: Insets::uniform(10.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 20.0)]);
        let out = grid(vec![TrackSize::Fixed(100.0)], vec![], 0.0, 0.0).layout(
            &style,
            &[child(1)],
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().height, 40.0);
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
            vec![
                TrackSize::Fixed(100.0),
                TrackSize::Fr(1.0),
                TrackSize::Fr(2.0),
            ],
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
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
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
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
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
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
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
        // Row 0 tallest = 35 → row 1 starts at y = 35.
        assert_eq!(b[2].origin(), Point::new(0.0, 35.0));
        assert_eq!(b[3].origin(), Point::new(100.0, 35.0));
        // Border box height = row0(35) + row1(15) = 50.
        assert_eq!(out.size(), Size::new(200.0, 50.0));
    }

    /// Builds a grid whose implicit tracks are sized by `grid-auto-columns` /
    /// `grid-auto-rows` (each cycled; empty → `auto`).
    fn grid_auto(
        columns: Vec<TrackSize>,
        rows: Vec<TrackSize>,
        auto_columns: Vec<TrackSize>,
        auto_rows: Vec<TrackSize>,
    ) -> GridLayout {
        GridLayout::with_template(
            GridTemplate::new(columns, rows).with_auto_tracks(auto_columns, auto_rows),
            0.0,
            0.0,
        )
    }

    // (auto-rows) explicit `100px 100px` columns, no explicit rows, and
    // `grid-auto-rows: 40px`: all rows are implicit and are 40px tall (NOT sized to
    // the content), so the two wrapped items land at y = 40.
    #[test]
    fn grid_auto_rows_sizes_implicit_rows() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3), child(4)];
        // Content heights (10) differ from the 40px track; the track wins.
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 4]);
        let g = grid_auto(
            vec![TrackSize::Fixed(100.0), TrackSize::Fixed(100.0)],
            vec![],
            vec![],
            vec![TrackSize::Fixed(40.0)],
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Row 0 items at y=0; wrapped row-1 items at y=40 (not the 10px content).
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[2].origin(), Point::new(0.0, 40.0));
        assert_eq!(b[3].origin(), Point::new(100.0, 40.0));
        assert_eq!(b[2].size().height, 40.0);
        // Two 40px implicit rows → border box height 80.
        assert_eq!(out.size(), Size::new(200.0, 80.0));
    }

    // (auto-columns) an item placed past the explicit columns (`grid-column: 4` in a
    // 2-column grid) opens implicit column 3, sized by `grid-auto-columns: 50px`.
    #[test]
    fn grid_auto_columns_sizes_implicit_column() {
        let style = LayoutStyle::default();
        let placement = GridPlacement::column(GridLine::Line(4), GridLine::Auto);
        let children = [placed(1, placement)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0)]);
        let g = grid_auto(
            vec![TrackSize::Fixed(100.0), TrackSize::Fixed(100.0)],
            vec![],
            vec![TrackSize::Fixed(50.0)],
            vec![],
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(1000.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Columns: 100,100 explicit then implicit 50 (col 2) + 50 (col 3). The item
        // sits in col 3 → x = 100+100+50 = 250, width 50.
        assert_eq!(b[0].origin().x, 250.0);
        assert_eq!(b[0].size().width, 50.0);
    }

    // (auto-rows cycling) `grid-auto-rows: 30px 50px` cycles across implicit rows:
    // row0=30, row1=50, row2=30 → single-column items stack at y 0/30/80.
    #[test]
    fn grid_auto_rows_cycles_two_sizes() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 3]);
        let g = grid_auto(
            vec![TrackSize::Fr(1.0)],
            vec![],
            vec![],
            vec![TrackSize::Fixed(30.0), TrackSize::Fixed(50.0)],
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().y, 0.0);
        assert_eq!(b[1].origin().y, 30.0); // after the 30px row 0
        assert_eq!(b[2].origin().y, 80.0); // after 30 + 50
        // Border box height = 30 + 50 + 30 = 110.
        assert_eq!(out.size().height, 110.0);
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

    // (p1) `30% 1fr` on a 400px content width: 30% = 120, fr takes the 280 remainder.
    #[test]
    fn percent_and_fr_columns_share_leftover() {
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Percent(0.30), TrackSize::Fr(1.0)],
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
        // 30% of 400 = 120; the 1fr track takes the 280 remainder. (f32 percent
        // arithmetic rounds 0.30 * 400 to ~120.00001, so compare with tolerance.)
        assert_eq!(b[0].origin().x, 0.0);
        assert!((b[0].size().width - 120.0).abs() < 1e-3);
        assert!((b[1].origin().x - 120.0).abs() < 1e-3);
        assert!((b[1].size().width - 280.0).abs() < 1e-3);
    }

    // (p2) two `50%` columns fill the content width exactly: 200 / 200 on 400px.
    #[test]
    fn two_percent_columns_split_content_width() {
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Percent(0.50), TrackSize::Percent(0.50)],
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
        assert_eq!(b[0].size().width, 200.0);
        assert_eq!(b[1].origin().x, 200.0);
        assert_eq!(b[1].size().width, 200.0);
    }

    // (p3) a `Percent` inside `minmax(30%, 1fr)` acts as the floor: with a 300px
    // fixed track ahead of it on a 400px width there is no room to grow, so the
    // minmax track floors at 30% of 400 = 120 (not below).
    #[test]
    fn minmax_percent_floor_holds() {
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![
                TrackSize::Fixed(300.0),
                TrackSize::minmax(TrackSize::Percent(0.30), TrackSize::Fr(1.0)),
            ],
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
        assert_eq!(b[0].size().width, 300.0);
        // No leftover for fr growth (300 + 120 > 400), so the minmax floors at 120
        // (f32 percent arithmetic rounds 0.30 * 400 to ~120.00001).
        assert_eq!(b[1].origin().x, 300.0);
        assert!((b[1].size().width - 120.0).abs() < 1e-3);
    }

    // (p4) percentage ROW tracks resolve against the container content height.
    #[test]
    fn percent_rows_resolve_against_content_height() {
        let style = LayoutStyle {
            height: Dimension::Px(200.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fr(1.0)],
            vec![TrackSize::Percent(0.25), TrackSize::Fr(1.0)],
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
        // Row 0 = 25% of 200 = 50; the second child starts on row 1 at y = 50.
        assert_eq!(b[1].origin().y, 50.0);
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
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
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
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
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
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
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

    // (neg-a) `grid-column: 1 / -1` spans every column: width 300 at x=0 (3×100).
    #[test]
    fn negative_end_line_spans_all_columns() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Line(1), GridLine::Line(-1)),
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
        assert_eq!(b[0].size().width, 300.0);
    }

    // (neg-b) `grid-column: -1` lands on the last column: x=200, width 100.
    #[test]
    fn negative_start_line_places_in_last_column() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::column(GridLine::Line(-1), GridLine::Auto),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0)]);
        let out = three_col_100().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 200.0);
        assert_eq!(b[0].size().width, 100.0);
    }

    // (neg-c) a negative row line: `grid-row: -1` places on the last explicit row.
    #[test]
    fn negative_row_line_places_on_last_row() {
        let style = LayoutStyle::default();
        let children = [placed(
            1,
            GridPlacement::row(GridLine::Line(-1), GridLine::Auto),
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 30.0)]);
        // Two explicit 50px rows; -1 is the last row (index 1) → y = 50.
        let g = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
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
        assert_eq!(b[0].origin().y, 50.0);
    }

    // ---- `repeat(auto-fill | auto-fit, …)` computed from the container size. ----

    /// A columns-only grid whose single axis is one `auto-fill`/`auto-fit` repeat.
    fn auto_repeat_grid(mode: AutoRepeatMode, pattern: Vec<TrackSize>, col_gap: f32) -> GridLayout {
        let columns = TrackList::new(Vec::new(), Some(AutoRepeat::new(mode, pattern)), Vec::new());
        GridLayout::with_template(
            GridTemplate::from_track_lists(columns, TrackList::default()),
            col_gap,
            0.0,
        )
    }

    fn width_style(px: f32) -> LayoutStyle {
        LayoutStyle {
            width: Dimension::Px(px),
            ..LayoutStyle::default()
        }
    }

    // (a) `repeat(auto-fill, 100px)` on a 350px container, no gap: 3 columns fit
    // (3*100 = 300 <= 350; a 4th overflows), placed at x = 0 / 100 / 200.
    #[test]
    fn auto_fill_fits_three_columns() {
        let style = width_style(350.0);
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 3]);
        let g = auto_repeat_grid(AutoRepeatMode::AutoFill, vec![TrackSize::Fixed(100.0)], 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 100.0);
        assert_eq!(b[1].origin().x, 100.0);
        assert_eq!(b[2].origin().x, 200.0);
    }

    // (a') the same count with a 10px gap: floor((350 + 10) / (100 + 10)) = 3,
    // columns at x = 0 / 110 / 220.
    #[test]
    fn auto_fill_count_accounts_for_gap() {
        let style = width_style(350.0);
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 3]);
        let g = auto_repeat_grid(
            AutoRepeatMode::AutoFill,
            vec![TrackSize::Fixed(100.0)],
            10.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[1].origin().x, 110.0);
        assert_eq!(b[2].origin().x, 220.0);
    }

    // (b) `auto-fit`: with fewer items than tracks, an empty interior track
    // collapses to 0 and its gaps collapse — an item on the last line packs left.
    // Container 350px, `repeat(auto-fit, 100px)` -> 3 tracks. Child A auto (col0),
    // child B on grid-column line 3 (col2); col1 is empty and collapses, so B sits
    // at x = 100 (not 200).
    #[test]
    fn auto_fit_collapses_empty_interior_track() {
        let style = width_style(350.0);
        let children = [
            child(1),
            placed(2, GridPlacement::column(GridLine::Line(3), GridLine::Auto)),
        ];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = auto_repeat_grid(AutoRepeatMode::AutoFit, vec![TrackSize::Fixed(100.0)], 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0); // A in col0
        assert_eq!(b[1].origin().x, 100.0); // B: col1 collapsed, so col2 starts at 100
    }

    // (b') `auto-fill` keeps the same empty interior track (no collapse): B on the
    // last line sits at x = 200 (col1 occupies its 100px).
    #[test]
    fn auto_fill_keeps_empty_interior_track() {
        let style = width_style(350.0);
        let children = [
            child(1),
            placed(2, GridPlacement::column(GridLine::Line(3), GridLine::Auto)),
        ];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = auto_repeat_grid(AutoRepeatMode::AutoFill, vec![TrackSize::Fixed(100.0)], 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[1].origin().x, 200.0); // col1 kept its 100px
    }

    // (c) a leading fixed 50px track ahead of `repeat(auto-fill, 100px)` on a 350px
    // container: 3 more 100px tracks fit the remaining 300px, so tracks are
    // [50, 100, 100, 100] and 4 children land at x = 0 / 50 / 150 / 250.
    #[test]
    fn auto_fill_count_after_leading_track() {
        let style = width_style(350.0);
        let children = [child(1), child(2), child(3), child(4)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 4]);
        let columns = TrackList::new(
            vec![TrackSize::Fixed(50.0)],
            Some(AutoRepeat::new(
                AutoRepeatMode::AutoFill,
                vec![TrackSize::Fixed(100.0)],
            )),
            Vec::new(),
        );
        let g = GridLayout::with_template(
            GridTemplate::from_track_lists(columns, TrackList::default()),
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(350.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 50.0);
        assert_eq!(b[1].origin().x, 50.0);
        assert_eq!(b[2].origin().x, 150.0);
        assert_eq!(b[3].origin().x, 250.0);
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

    // ---- `grid-auto-flow`: dense packing + column flow. ----

    /// Three span-then-1x1 items in a 3-column grid leave a hole at `(row0, col2)`:
    /// two span-2 items land on row 0 and row 1, then a 1x1 item follows. `dense`
    /// backfills the hole (row 0); the default sparse flow does not (row 1). The
    /// two flows must place that item at *different* coordinates.
    fn hole_children() -> [LayoutChild; 3] {
        [
            placed(1, GridPlacement::column(GridLine::Auto, GridLine::Span(2))),
            placed(2, GridPlacement::column(GridLine::Auto, GridLine::Span(2))),
            child(3),
        ]
    }

    #[test]
    fn dense_backfills_earlier_hole_but_sparse_does_not() {
        let cols = || {
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
        };
        let sizes = || {
            FixedChildMeasurer::new(vec![
                Size::new(0.0, 10.0),
                Size::new(0.0, 10.0),
                Size::new(0.0, 10.0),
            ])
        };
        let constraints = BoxConstraints::loose(Size::new(300.0, 1000.0));

        // Sparse (default `Row`): the 1x1 item does NOT backfill — it follows the
        // forward cursor onto row 1, col 2.
        let sparse = LayoutStyle::default();
        let mut m1 = sizes();
        let out_sparse = cols().layout(&sparse, &hole_children(), constraints, &mut m1);
        let bs = out_sparse.child_boxes();
        assert_eq!(bs[2].origin(), Point::new(200.0, 10.0));

        // Dense (`RowDense`): the 1x1 item backfills the hole at row 0, col 2.
        let dense = LayoutStyle::default().with_grid_auto_flow(GridAutoFlow::RowDense);
        let mut m2 = sizes();
        let out_dense = cols().layout(&dense, &hole_children(), constraints, &mut m2);
        let bd = out_dense.child_boxes();
        assert_eq!(bd[2].origin(), Point::new(200.0, 0.0));

        // The two flows differ (the whole point of `dense`).
        assert_ne!(bs[2].origin(), bd[2].origin());
    }

    /// `column` flow fills *down* the rows of column 0, then across to column 1:
    /// with two fixed 20px rows, four 1x1 items land at
    /// `(c0,r0) (c0,r1) (c1,r0) (c1,r1)`.
    #[test]
    fn column_flow_places_down_then_across() {
        let style = LayoutStyle::default().with_grid_auto_flow(GridAutoFlow::Column);
        let children = [child(1), child(2), child(3), child(4)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 20.0); 4]);
        let g = grid(
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
            vec![TrackSize::Fixed(20.0), TrackSize::Fixed(20.0)],
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
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0)); // col0, row0
        assert_eq!(b[1].origin(), Point::new(0.0, 20.0)); // col0, row1 (down)
        assert_eq!(b[2].origin(), Point::new(50.0, 0.0)); // col1, row0 (across)
        assert_eq!(b[3].origin(), Point::new(50.0, 20.0)); // col1, row1
    }

    /// Regression: the default (`Row`, sparse) flow is unchanged — the same
    /// row-major result as before `grid-auto-flow` existed.
    #[test]
    fn default_row_sparse_is_unchanged() {
        let style = LayoutStyle::default();
        assert_eq!(style.grid_auto_flow(), GridAutoFlow::Row);
        let children = [child(1), child(2), child(3), child(4), child(5)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 5]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
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
        assert_eq!(b[3].origin(), Point::new(100.0, 10.0)); // col1,row1
        assert_eq!(b[4].origin(), Point::new(0.0, 20.0)); // col0,row2
        assert_eq!(out.size(), Size::new(200.0, 30.0));
    }

    // ---- Grid item alignment within a cell (`justify-self` / `align-self`). ----

    use crate::model::{AlignItems, AlignSelf};

    /// A single `40×30` child in one `100×100` fixed cell, aligned per-item.
    fn aligned_in_cell(justify: AlignSelf, align: AlignSelf) -> (f32, f32, f32, f32) {
        aligned_in_cell_with(LayoutStyle::default(), justify, align)
    }

    /// As [`aligned_in_cell`], but the container `style` supplies the
    /// `justify-items` / `align-items` defaults.
    fn aligned_in_cell_with(
        style: LayoutStyle,
        justify: AlignSelf,
        align: AlignSelf,
    ) -> (f32, f32, f32, f32) {
        let child = LayoutChild::new(
            NodeId::from_raw(1),
            LayoutStyle::default()
                .with_justify_self(justify)
                .with_align_self(align),
        );
        // The measurer reports the item's natural size (40×30) regardless of
        // constraints; the fixed 100×100 tracks are the cell.
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(40.0, 30.0)]);
        let g = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::Fixed(100.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &[child],
            BoxConstraints::loose(Size::new(100.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes()[0];
        (b.origin().x, b.origin().y, b.size().width, b.size().height)
    }

    // `justify-self:start` + `align-self:start` → the item sits at the cell
    // origin at its natural 40×30 size (no stretch to the 100×100 cell).
    #[test]
    fn self_start_places_at_cell_origin_natural_size() {
        assert_eq!(
            aligned_in_cell(AlignSelf::Start, AlignSelf::Start),
            (0.0, 0.0, 40.0, 30.0)
        );
    }

    // `center` on both axes → offset ((100-40)/2, (100-30)/2) = (30, 35).
    #[test]
    fn self_center_offsets_by_half_the_slack() {
        assert_eq!(
            aligned_in_cell(AlignSelf::Center, AlignSelf::Center),
            (30.0, 35.0, 40.0, 30.0)
        );
    }

    // `end` on both axes → offset (100-40, 100-30) = (60, 70).
    #[test]
    fn self_end_places_at_cell_end() {
        assert_eq!(
            aligned_in_cell(AlignSelf::End, AlignSelf::End),
            (60.0, 70.0, 40.0, 30.0)
        );
    }

    // `stretch` (the default) → the item fills the 100×100 cell, byte-identical
    // to the historical grid behaviour.
    #[test]
    fn self_stretch_fills_the_cell() {
        assert_eq!(
            aligned_in_cell(AlignSelf::Stretch, AlignSelf::Stretch),
            (0.0, 0.0, 100.0, 100.0)
        );
        // The `auto`/`auto` default (no per-item override, container all-stretch)
        // is likewise a full-cell fill.
        assert_eq!(
            aligned_in_cell(AlignSelf::Auto, AlignSelf::Auto),
            (0.0, 0.0, 100.0, 100.0)
        );
    }

    // The two axes are independent: `justify-self:end` (inline/x) with
    // `align-self:stretch` (block/y) → x at the cell end, y filling the cell.
    #[test]
    fn self_axes_are_independent() {
        assert_eq!(
            aligned_in_cell(AlignSelf::End, AlignSelf::Stretch),
            (60.0, 0.0, 40.0, 100.0)
        );
    }

    // A container `justify-items:center` default applies to an item whose
    // `justify-self` is `auto`; the block axis stays at the container's
    // `align-items` (stretch) default.
    #[test]
    fn container_justify_items_applies_when_self_is_auto() {
        let container = LayoutStyle::default().with_justify_items(AlignItems::Center);
        assert_eq!(
            aligned_in_cell_with(container, AlignSelf::Auto, AlignSelf::Auto),
            (30.0, 0.0, 40.0, 100.0)
        );
    }

    // ---- `stretch` against an item that has a size of its own ----------------
    //
    // `specs/w3c/css-align-3.txt:1377`: `stretch` sets the used size only "when
    // the box's computed width/height (as appropriate to the axis) is auto", and
    // otherwise "falls back to flex-start generally". Every case below used to
    // come back `(0, 0, 100, 100)` — the whole cell — with the declared size
    // thrown away (`docs/external-consumer-audit.md`, 2-4 (b)).

    /// As [`aligned_in_cell`], but the item declares a size of its own on one or
    /// both axes. The natural size stays `40×30`, so a declared `40`/`30` is
    /// exactly what a flex-start fallback should leave the box at, and the old
    /// full-cell `100` is exactly what it should no longer be.
    fn sized_in_cell(
        width: Dimension,
        height: Dimension,
        justify: AlignSelf,
        align: AlignSelf,
    ) -> (f32, f32, f32, f32) {
        let child = LayoutChild::new(
            NodeId::from_raw(1),
            LayoutStyle::default()
                .with_width(width)
                .with_height(height)
                .with_justify_self(justify)
                .with_align_self(align),
        );
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(40.0, 30.0)]);
        let out = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::Fixed(100.0)],
            0.0,
            0.0,
        )
        .layout(
            &LayoutStyle::default(),
            &[child],
            BoxConstraints::loose(Size::new(100.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes()[0];
        (b.origin().x, b.origin().y, b.size().width, b.size().height)
    }

    /// A definite **width** takes the inline axis out of `stretch`: the item keeps
    /// its 40 and sits at the cell's inline start. The block axis is untouched —
    /// `height` is still `auto`, so it still stretches to the full 100.
    #[test]
    fn a_definite_width_falls_back_to_flex_start_on_the_inline_axis_only() {
        assert_eq!(
            sized_in_cell(
                Dimension::Px(40.0),
                Dimension::Auto,
                AlignSelf::Auto,
                AlignSelf::Auto
            ),
            (0.0, 0.0, 40.0, 100.0)
        );
    }

    /// And symmetrically on the block axis — the spec's condition is written "as
    /// appropriate to the axis", so a definite `height` must be honoured the same
    /// way a definite `width` is. The inline axis keeps stretching.
    #[test]
    fn a_definite_height_falls_back_to_flex_start_on_the_block_axis_only() {
        assert_eq!(
            sized_in_cell(
                Dimension::Auto,
                Dimension::Px(30.0),
                AlignSelf::Auto,
                AlignSelf::Auto
            ),
            (0.0, 0.0, 100.0, 30.0)
        );
    }

    /// An author who writes `justify-self: stretch` / `align-self: stretch` by
    /// hand on a sized item gets exactly what the inherited default gives: the
    /// spec's condition is about the box's computed size, not about how the
    /// alignment value was arrived at. Writing the keyword out loud does not buy a
    /// stretch the keyword is not defined to perform.
    #[test]
    fn an_authored_stretch_on_a_sized_item_falls_back_the_same_way() {
        assert_eq!(
            sized_in_cell(
                Dimension::Px(40.0),
                Dimension::Px(30.0),
                AlignSelf::Stretch,
                AlignSelf::Stretch
            ),
            (0.0, 0.0, 40.0, 30.0),
            "an authored `stretch`"
        );
        assert_eq!(
            sized_in_cell(
                Dimension::Px(40.0),
                Dimension::Px(30.0),
                AlignSelf::Auto,
                AlignSelf::Auto
            ),
            (0.0, 0.0, 40.0, 30.0),
            "and the inherited default — the same box"
        );
    }

    /// An intrinsic keyword is not `auto` either, so it falls back too. That is
    /// the right answer and not an accident: `width: min-content` asks for the
    /// content's size, which is precisely what a stretch would overwrite.
    #[test]
    fn an_intrinsic_size_is_not_auto_and_so_is_not_stretched() {
        assert_eq!(
            sized_in_cell(
                Dimension::MinContent,
                Dimension::Auto,
                AlignSelf::Auto,
                AlignSelf::Auto
            ),
            (0.0, 0.0, 40.0, 100.0)
        );
    }

    /// A sized item that is *aligned* rather than stretched is unmoved by any of
    /// this: `end` still puts its declared 40×30 at the cell end. The fallback
    /// only ever replaces `stretch`.
    #[test]
    fn an_explicit_alignment_on_a_sized_item_is_untouched() {
        assert_eq!(
            sized_in_cell(
                Dimension::Px(40.0),
                Dimension::Px(30.0),
                AlignSelf::End,
                AlignSelf::End
            ),
            (60.0, 70.0, 40.0, 30.0)
        );
    }

    /// The fallback reaches the *measure* pass too, not just the placement.
    ///
    /// Row sizing feeds on each item's measured height, and that measurement is
    /// only meaningful at the inline size the item will actually be laid out at.
    /// While `stretch` was unconditional the two agreed by luck — every item was
    /// measured at the full track and placed at the full track. A definite width
    /// breaks that tie, so the offer has to follow: a stretched item is still
    /// offered a tight `min.width == max.width == track`, and one that falls back
    /// to flex-start is offered `min.width == 0` and left to be its own width.
    #[test]
    fn a_flex_start_fallback_is_offered_a_loose_inline_axis_to_measure_in() {
        #[derive(Default)]
        struct RecordOffer(Vec<BoxConstraints>);
        impl ChildMeasurer for RecordOffer {
            fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
                self.0.push(constraints);
                Size::new(40.0, 30.0)
            }
        }
        let sized = |width: Dimension| {
            let mut measurer = RecordOffer::default();
            grid(
                vec![TrackSize::Fixed(100.0)],
                vec![TrackSize::Fixed(100.0)],
                0.0,
                0.0,
            )
            .layout(
                &LayoutStyle::default(),
                &[LayoutChild::new(
                    NodeId::from_raw(1),
                    LayoutStyle::default().with_width(width),
                )],
                BoxConstraints::loose(Size::new(100.0, 100.0)),
                &mut measurer,
            );
            measurer.0[0].min.width
        };
        assert_eq!(sized(Dimension::Auto), 100.0, "still stretched: tight");
        assert_eq!(sized(Dimension::Px(40.0)), 0.0, "flex-start: shrink to fit");
    }

    // A per-item `justify-self` overrides the container `justify-items` default.
    #[test]
    fn self_overrides_container_items_default() {
        let container = LayoutStyle::default().with_justify_items(AlignItems::Center);
        // `justify-self:start` beats the container `center`.
        assert_eq!(
            aligned_in_cell_with(container, AlignSelf::Start, AlignSelf::Auto),
            (0.0, 0.0, 40.0, 100.0)
        );
    }

    // ---- content distribution: `justify-content` (columns) / `align-content`
    // (rows) distribute the leftover free space when the tracks are smaller than
    // the container content extent. Two 50px tracks in a 200px container leave a
    // 100px leftover.

    /// Lays out two `50px` columns in a 200px-wide container with `justify` and
    /// returns the two children's `x` offsets.
    fn distribute_cols(justify: JustifyContent) -> (f32, f32) {
        let style = LayoutStyle {
            width: Dimension::Px(200.0),
            justify_content: justify,
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 2]);
        let g = grid(
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
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
        (b[0].origin().x, b[1].origin().x)
    }

    /// Lays out two `50px` rows (single column) in a 200px-tall container with
    /// `align` and returns the two children's `y` offsets.
    fn distribute_rows(align: JustifyContent) -> (f32, f32) {
        let style = LayoutStyle {
            height: Dimension::Px(200.0),
            align_content: align,
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 2]);
        let g = grid(
            vec![TrackSize::Fr(1.0)],
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 200.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        (b[0].origin().y, b[1].origin().y)
    }

    #[test]
    fn justify_content_start_is_unshifted_regression() {
        // The default packs the columns at the start — byte-identical to before.
        assert_eq!(distribute_cols(JustifyContent::FlexStart), (0.0, 50.0));
    }

    #[test]
    fn justify_content_end_shifts_by_full_leftover() {
        assert_eq!(distribute_cols(JustifyContent::FlexEnd), (100.0, 150.0));
    }

    #[test]
    fn justify_content_center_shifts_by_half_leftover() {
        assert_eq!(distribute_cols(JustifyContent::Center), (50.0, 100.0));
    }

    #[test]
    fn justify_content_space_between_puts_leftover_between_tracks() {
        // n = 2 → the whole 100px leftover becomes the gap between the tracks.
        assert_eq!(distribute_cols(JustifyContent::SpaceBetween), (0.0, 150.0));
    }

    #[test]
    fn justify_content_space_around_half_gaps_at_ends() {
        // leftover/(2n) = 25 at the start, leftover/n = 50 between the tracks.
        assert_eq!(distribute_cols(JustifyContent::SpaceAround), (25.0, 125.0));
    }

    #[test]
    fn justify_content_space_evenly_equal_gaps_including_ends() {
        // leftover/(n+1) = 100/3 ≈ 33.33 at each of the three gaps.
        let (x0, x1) = distribute_cols(JustifyContent::SpaceEvenly);
        assert!((x0 - 100.0 / 3.0).abs() < 1e-3, "x0 = {x0}");
        assert!(
            (x1 - (100.0 / 3.0 + 50.0 + 100.0 / 3.0)).abs() < 1e-3,
            "x1 = {x1}"
        );
    }

    #[test]
    fn align_content_distributes_rows_on_the_block_axis() {
        // The same math on the row (block) axis via `align-content`.
        assert_eq!(distribute_rows(JustifyContent::FlexStart), (0.0, 50.0));
        assert_eq!(distribute_rows(JustifyContent::FlexEnd), (100.0, 150.0));
        assert_eq!(distribute_rows(JustifyContent::Center), (50.0, 100.0));
        assert_eq!(distribute_rows(JustifyContent::SpaceBetween), (0.0, 150.0));
        assert_eq!(distribute_rows(JustifyContent::SpaceAround), (25.0, 125.0));
        let (y0, y1) = distribute_rows(JustifyContent::SpaceEvenly);
        assert!((y0 - 100.0 / 3.0).abs() < 1e-3, "y0 = {y0}");
        assert!(
            (y1 - (100.0 / 3.0 + 50.0 + 100.0 / 3.0)).abs() < 1e-3,
            "y1 = {y1}"
        );
    }

    #[test]
    fn content_distribution_axes_are_independent() {
        // `justify-content: end` on columns must not disturb the row (block) axis,
        // which keeps its `align-content: start` default.
        let style = LayoutStyle {
            width: Dimension::Px(200.0),
            height: Dimension::Px(200.0),
            justify_content: JustifyContent::FlexEnd,
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(0.0, 10.0); 2]);
        // 50px 50px columns, one 50px row → both items in row 0.
        let g = grid(
            vec![TrackSize::Fixed(50.0), TrackSize::Fixed(50.0)],
            vec![TrackSize::Fixed(50.0)],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 200.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Columns shifted to the end (x = 100 / 150); rows unshifted (y = 0).
        assert_eq!(b[0].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(150.0, 0.0));
    }

    // ---- Canonical §11 track-sizing pipeline: the four divergence fixes plus a
    // regression that the simple even-split path is unchanged. ----

    // (fr-restart, §11.7) `minmax(200px,1fr) 1fr` in a 300px container. Track 0's
    // fr share (150) is below its 200px floor, so it pins at 200 and its UNUSED
    // fr share flows to track 1, which takes ALL the remaining 100 (not a 50/50
    // split). The old single-pass fr gave track 1 only 50.
    #[test]
    fn fr_restart_gives_unused_share_to_other_flex_track() {
        let style = LayoutStyle {
            width: Dimension::Px(300.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![
                TrackSize::minmax(TrackSize::Fixed(200.0), TrackSize::Fr(1.0)),
                TrackSize::Fr(1.0),
            ],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 200.0);
        assert_eq!(b[1].origin().x, 200.0);
        assert_eq!(b[1].size().width, 100.0);
    }

    // (span distribution, §11.5) `auto auto` with a colspan-2 item 200px wide:
    // its content is shared ACROSS both auto tracks (100 each), not attributed to
    // the first spanned track alone. A following 1×1 item in column 0 fills that
    // 100px cell — proving column 0 is 100 (the old code sized it 200).
    #[test]
    fn spanning_item_shares_content_across_auto_tracks() {
        let style = LayoutStyle::default();
        let children = [
            placed(1, GridPlacement::column(GridLine::Auto, GridLine::Span(2))),
            child(2),
        ];
        // Item 0 is 200px wide (crosses both auto cols); item 1 is zero-width.
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(200.0, 20.0), Size::new(0.0, 10.0)]);
        let g = grid(vec![TrackSize::Auto, TrackSize::Auto], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(1000.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // The span-2 item covers both 100px tracks → 200 wide at x = 0.
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 200.0);
        // The 1×1 item lands in column 0 (row 1) and fills its cell: width 100
        // confirms the 200px content was split 100/100, not dumped on column 0.
        assert_eq!(b[1].origin().x, 0.0);
        assert_eq!(b[1].size().width, 100.0);
    }

    // (maximise, §11.6) `minmax(100px,300px)` alone in a 500px container grows to
    // its 300px growth limit (free space is distributed up to the limit). The old
    // model left it at its 100px floor.
    #[test]
    fn maximise_grows_minmax_to_its_growth_limit() {
        let style = LayoutStyle {
            width: Dimension::Px(500.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(50.0, 20.0)]);
        let g = grid(
            vec![TrackSize::minmax(
                TrackSize::Fixed(100.0),
                TrackSize::Fixed(300.0),
            )],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 300.0);
    }

    // (stretch, §11.8) `auto auto` in a definite 400px container with small
    // content: with the default (`normal`/`stretch`) content alignment the two
    // auto tracks each stretch to fill 200px. The old model packed them at the
    // start at their content size.
    #[test]
    fn stretch_expands_auto_tracks_to_fill_definite_container() {
        let style = LayoutStyle {
            width: Dimension::Px(400.0),
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(30.0, 10.0), Size::new(30.0, 10.0)]);
        let g = grid(vec![TrackSize::Auto, TrackSize::Auto], vec![], 0.0, 0.0);
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(400.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin().x, 0.0);
        assert_eq!(b[0].size().width, 200.0);
        assert_eq!(b[1].origin().x, 200.0);
        assert_eq!(b[1].size().width, 200.0);
    }

    // (regression) a plain `1fr 1fr` even split is unchanged by the new pipeline:
    // two equal halves of the 300px content width.
    #[test]
    fn even_fr_split_is_unchanged_by_pipeline() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(0.0, 10.0), Size::new(0.0, 10.0)]);
        let g = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        );
        let out = g.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 150.0);
        assert_eq!(b[1].origin().x, 150.0);
        assert_eq!(b[1].size().width, 150.0);
    }

    // ---- min-content / max-content tracks (audit Tier-1 §4-D) --------------

    /// An item that answers the offer like a paragraph: it fills the inline space
    /// it is given, never below `min` (its longest unbreakable word) and never
    /// above `max` (its whole content on one line). [`FixedChildMeasurer`] cannot
    /// catch a probe that is not probing — it returns the same size whatever it is
    /// offered — so these tests need a measurer with an opinion.
    struct Paragraph {
        min: f32,
        max: f32,
    }

    impl crate::algorithm::ChildMeasurer for Paragraph {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            Size::new(constraints.max.width.clamp(self.min, self.max), 10.0)
        }
    }

    /// The single column's width under `track`, in a `line`-wide grid holding one
    /// paragraph item (min-content 30, max-content 200).
    fn one_track_width(track: TrackSize, line: f32) -> f32 {
        let out = grid(vec![track], vec![], 0.0, 0.0).layout(
            &LayoutStyle::default(),
            &[child(1)],
            BoxConstraints::loose(Size::new(line, 1000.0)),
            &mut Paragraph {
                min: 30.0,
                max: 200.0,
            },
        );
        out.child_boxes()[0].size().width
    }

    #[test]
    fn a_min_content_column_is_the_widest_unbreakable_run() {
        // The probe offers the item no inline space, so it reports the 30 it cannot
        // break below — NOT the 100 the container has to give.
        assert_eq!(one_track_width(TrackSize::MinContent, 100.0), 30.0);
    }

    #[test]
    fn a_max_content_column_is_the_content_on_one_line() {
        // Unbounded offer → the item's whole content on one line (200), which
        // OVERFLOWS the 100px container: `max-content` is a question about the
        // content, and the container's answer is not it.
        assert_eq!(one_track_width(TrackSize::MaxContent, 100.0), 200.0);
    }

    #[test]
    fn min_content_max_content_and_auto_columns_all_disagree() {
        // If any two of these agreed, the probes would not be probing. The 100px
        // container is the case that separates them: `auto` takes the space it has
        // (the documented single-measure approximation), while the two keywords
        // each report a different property of the content and ignore the container.
        let min = one_track_width(TrackSize::MinContent, 100.0);
        let max = one_track_width(TrackSize::MaxContent, 100.0);
        let auto = one_track_width(TrackSize::Auto, 100.0);
        assert_eq!((min, max, auto), (30.0, 200.0, 100.0));
    }

    #[test]
    fn a_min_content_column_beside_an_fr_gives_the_fr_the_rest() {
        // The pair that makes the keyword worth having: a content-hugging label
        // column beside a flexible one. The label takes its 30, the `1fr` takes
        // the other 270.
        let out = grid(
            vec![TrackSize::MinContent, TrackSize::Fr(1.0)],
            vec![],
            0.0,
            0.0,
        )
        .layout(
            &LayoutStyle::default(),
            &[child(1), child(2)],
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut Paragraph {
                min: 30.0,
                max: 200.0,
            },
        );
        let b = out.child_boxes();
        assert_eq!(b[0].size().width, 30.0);
        assert_eq!(b[1].origin().x, 30.0);
        assert_eq!(b[1].size().width, 270.0);
    }

    #[test]
    fn a_minmax_min_content_floor_is_the_min_content_size_not_the_auto_one() {
        // `minmax(min-content, 1fr) 100px` in a 200px grid. The floor is the item's
        // MIN-content size (30), which the `1fr` then covers — so the flexible
        // track gets its 100 and the grid fits. Were the floor `auto`'s single
        // loose measure (200, the space the container has), the base would exceed
        // the `fr` share and the grid would overflow to 200 + 100. The `minmax`
        // recursion is what has to carry the keyword down to the min bound.
        let out = grid(
            vec![
                TrackSize::minmax(TrackSize::MinContent, TrackSize::Fr(1.0)),
                TrackSize::Fixed(100.0),
            ],
            vec![],
            0.0,
            0.0,
        )
        .layout(
            &LayoutStyle::default(),
            &[child(1), child(2)],
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut Paragraph {
                min: 30.0,
                max: 200.0,
            },
        );
        let b = out.child_boxes();
        assert_eq!(
            b[0].size().width,
            100.0,
            "1fr of the 100 left after the 100px"
        );
        assert_eq!(b[1].origin().x, 100.0, "the fixed track is not pushed out");
    }

    #[test]
    fn a_content_keyword_row_takes_the_items_height() {
        // The block axis: with its width fixed to the column, the item's three
        // intrinsic heights coincide, so a `min-content` row is its measured
        // height — 10 here — and not the 1000 the container offers.
        let out = grid(
            vec![TrackSize::Fixed(100.0)],
            vec![TrackSize::MinContent],
            0.0,
            0.0,
        )
        .layout(
            &LayoutStyle::default(),
            &[child(1)],
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut Paragraph {
                min: 30.0,
                max: 200.0,
            },
        );
        assert_eq!(out.child_boxes()[0].size().height, 10.0);
    }

    // ---- the containing block a grid item hands to its children -------------

    /// A `width: 50%` block — the child the external-consumer audit measured
    /// inside every row of its table.
    fn half_width() -> LayoutStyle {
        LayoutStyle {
            width: Dimension::Percent(50.0),
            ..LayoutStyle::default()
        }
    }

    /// Lays one nested block flow box out under whatever constraints it is handed
    /// — the leaf of the audit's shell.
    struct NestedBlock(LayoutStyle);

    impl ChildMeasurer for NestedBlock {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            FlowLayout
                .layout(
                    &self.0,
                    &[],
                    constraints,
                    &mut FixedChildMeasurer::default(),
                )
                .size()
        }
    }

    /// Measures a grid item that is a block flow box carrying one `width: 50%`
    /// child, so the item really does have a subtree to hand a containing block to.
    struct GridItem(LayoutStyle);

    impl ChildMeasurer for GridItem {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            FlowLayout
                .layout(
                    &self.0,
                    &[LayoutChild::new(NodeId::from_raw(9), half_width())],
                    constraints,
                    &mut NestedBlock(half_width()),
                )
                .size()
        }
    }

    /// The audit's shell, laid out directly: `width/height: 100%`,
    /// `box-sizing: border-box`, `padding: 24px`, `grid-template-columns: 1fr 1fr`,
    /// `gap: 10px` at 480×240 — a 432-wide content box and two 211 tracks.
    fn audit_shell() -> LayoutStyle {
        LayoutStyle {
            width: Dimension::Percent(100.0),
            height: Dimension::Percent(100.0),
            box_sizing: BoxSizing::BorderBox,
            padding: Insets::uniform(24.0),
            ..LayoutStyle::default()
        }
    }

    /// One row of the audit's table, run through the two passes the runtime runs:
    /// the grid arranges its item, then the item is laid out again at the box the
    /// grid gave it, under the containing block the grid **stated**. Returns
    /// `(the item's used width, its child's)`.
    fn audit_row(item_width: Dimension) -> (f32, f32) {
        let item_style = LayoutStyle {
            width: item_width,
            ..LayoutStyle::default()
        };
        let out = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            10.0,
            10.0,
        )
        .layout(
            &audit_shell(),
            // The item's style reaches the algorithm through its `LayoutChild`,
            // exactly as the runtime hands it over (`children_of` in the layout
            // coordinator resolves each child's style into the descriptor). The
            // measurer carries the same style because it is the same box: the
            // algorithm reads the declared width from the descriptor, and the
            // measure re-lays it out.
            &[LayoutChild::new(NodeId::from_raw(1), item_style)],
            BoxConstraints::tight(Size::new(480.0, 240.0)),
            &mut GridItem(item_style),
        );
        let item_box = out.child_boxes()[0].size();
        // The caller's fallback basis is the grid container's own content box —
        // 480×240 border-box less 24px of padding on each edge.
        let basis = out.child_percent_basis(0).resolve(Size::new(432.0, 192.0));
        let item_out = FlowLayout.layout(
            &item_style,
            &[LayoutChild::new(NodeId::from_raw(9), half_width())],
            BoxConstraints::tight(item_box).with_percent_basis(basis),
            &mut NestedBlock(half_width()),
        );
        (item_box.width, item_out.child_boxes()[0].size().width)
    }

    /// Every item is handed its **grid area** as its percentage basis, on **both**
    /// axes.
    ///
    /// `specs/w3c/css-grid-2.txt:712`: "A grid item's grid area forms the
    /// containing block into which it is laid out." Two `1fr` columns in a 432
    /// content box with a 10px gap are 211 each, so both items say 211 — not the
    /// 432 the container would otherwise hand down.
    ///
    /// The block component is `Some(10.0)`, the content-sized row's extent, and an
    /// earlier round of this fix left it `None` on the worry that a box cannot be
    /// its own containing block. `specs/w3c/css-grid-2.txt:3527` settles it: "Lay
    /// out the grid items into their respective containing blocks. Each grid
    /// area's width and height are considered definite for this purpose." The
    /// measure that sized the row has already finished before any item is placed,
    /// so the number is a resolved output, not an input to itself.
    #[test]
    fn each_item_states_its_grid_area_as_its_percentage_basis_on_both_axes() {
        let out = grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            10.0,
            10.0,
        )
        .layout(
            &audit_shell(),
            &[child(1), child(2)],
            BoxConstraints::tight(Size::new(480.0, 240.0)),
            &mut FixedChildMeasurer::new(vec![Size::new(10.0, 10.0), Size::new(10.0, 10.0)]),
        );
        assert_eq!(
            out.child_percent_basis(0),
            ChildPercentBasis::with_size(211.0, 10.0)
        );
        assert_eq!(
            out.child_percent_basis(1),
            ChildPercentBasis::with_size(211.0, 10.0)
        );
    }

    /// The external-consumer audit's 2-4 table, reproduced on the algorithms
    /// themselves.
    ///
    /// The `100%` row is the defect: the item's used width was right (211, its
    /// track) while its child came back **216** — 50% of 432, the grid
    /// *container's* content box, a containing block the item never had. The
    /// `auto` row is the control and never moved.
    ///
    /// The `50px` row was the audit's other, separate defect (2-4 (b)): the item
    /// was stretched to the whole track although `stretch` applies only when the
    /// box's computed size "is auto" (`specs/w3c/css-align-3.txt:1377`). It now
    /// keeps its declared 50 and sits at the start of its area, which is what
    /// "falls back to flex-start" means; its child, `50%` of that, is 25.
    ///
    /// The audit's own "child should be" column reads 105.5 on all three rows,
    /// which is right for the two rows whose item is a whole track wide and is a
    /// column heading carried down on the third: an item that is genuinely 50px
    /// wide cannot hand a `50%` child anything but 25. The audit's prose says the
    /// same — in the `50px` row "the child is arithmetically consistent with the
    /// declared width while the item is not". So the row's oracle is (50, 25).
    #[test]
    fn a_grid_items_children_resolve_against_the_items_grid_area() {
        assert_eq!(audit_row(Dimension::Auto), (211.0, 105.5), "the control");
        assert_eq!(
            audit_row(Dimension::Percent(100.0)),
            (211.0, 105.5),
            "was (211, 216): the child had resolved against the container's 432"
        );
        assert_eq!(
            audit_row(Dimension::Px(50.0)),
            (50.0, 25.0),
            "was (211, 25): `stretch` had run over the item's declared width"
        );
    }

    /// Stating the item's containing block did not change what the item is
    /// **offered**: the measure pass still hands every item its track width as a
    /// tight inline range and the container's content height as the block bound.
    /// The offer and the basis are separate facts, and this pins that separation —
    /// which is the whole reason `BoxConstraints` carries them apart.
    ///
    /// The basis itself is `(211, ∞)`: the inline component is the item's grid
    /// area, and the block component is infinite because at this point in the
    /// algorithm the rows have not been sized — they are sized *from* this
    /// measurement — so the item's block axis is genuinely indefinite and a
    /// percentage there must behave as `auto` (`specs/w3c/css-sizing-3.txt:554`).
    /// It read `None` before, which made the fallback quietly use `max.height`, the
    /// container's content height, as if it were a containing block.
    #[test]
    fn stating_the_basis_did_not_change_the_offer() {
        #[derive(Default)]
        struct RecordOffer(Vec<BoxConstraints>);
        impl ChildMeasurer for RecordOffer {
            fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
                self.0.push(constraints);
                Size::new(constraints.max.width, 10.0)
            }
        }
        let mut measurer = RecordOffer::default();
        grid(
            vec![TrackSize::Fr(1.0), TrackSize::Fr(1.0)],
            vec![],
            10.0,
            10.0,
        )
        .layout(
            &audit_shell(),
            &[child(1)],
            BoxConstraints::tight(Size::new(480.0, 240.0)),
            &mut measurer,
        );
        let sizing = measurer.0.last().expect("the item was measured");
        assert_eq!(sizing.min, Size::new(211.0, 0.0));
        assert_eq!(sizing.max, Size::new(211.0, 192.0));
        assert_eq!(
            sizing.percent_basis,
            Some(Size::new(211.0, f32::INFINITY)),
            "the area on the inline axis; indefinite on the block axis, where the \
             rows are not sized yet"
        );
    }
}
