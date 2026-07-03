//! `GridLayout`: a uniform fixed-column grid formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{LayoutBox, LayoutStyle};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// Uniform fixed-column grid layout (spec §4.1, §4.2).
///
/// Children flow **row-major** into a grid with a fixed number of equal-width
/// columns: child `0` fills column `0` of row `0`, child `1` column `1`, and so
/// on, wrapping to a new row once every column in the current row is filled.
/// Every column has the same width; a row is as tall as its tallest child.
///
/// # Configuration
///
/// This is a **self-configured** algorithm (the model carries no grid-template
/// fields yet): the number of `columns`, the horizontal `column_gap`, and the
/// vertical `row_gap` are stored on the value. `columns` is clamped to at least
/// `1` at construction.
///
/// # Algorithm
///
/// 1. **Content area** — the container `padding` is subtracted from
///    `constraints.max` to get the content width/height (each clamped to be
///    non-negative).
/// 2. **Column width** — `(content_width - (columns - 1) * column_gap) /
///    columns`, clamped to be non-negative. Every cell shares this width.
/// 3. **Placement** — walking children in order, child at index `i` sits in
///    column `i % columns` of row `i / columns`. Each child is measured under a
///    [`BoxConstraints`] fixing its width to the cell width and allowing its
///    natural height up to the remaining content height, then placed at
///    `(padding.left + col * (cell_width + column_gap), padding.top + row_top)`
///    where `row_top` accumulates each earlier row's height plus `row_gap`.
/// 4. **Row height** — a row is as tall as the tallest child measured in it; the
///    next row starts below it (plus `row_gap`).
/// 5. **Border box** — the bounding box of the placed children plus the
///    container `padding`, then clamped into `constraints`.
///
/// # Uniform-grid simplification
///
/// This is deliberately a **uniform** grid: fixed equal-width columns and
/// content-sized rows. Full CSS grid tracks — named lines, `fr` units, explicit
/// row/column templates, spanning, auto-placement dense packing, and alignment
/// within tracks — are future work and are not modelled here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLayout {
    /// Number of equal-width columns; always at least `1`.
    columns: usize,
    /// Horizontal gap between adjacent columns, in logical pixels.
    column_gap: f32,
    /// Vertical gap between adjacent rows, in logical pixels.
    row_gap: f32,
}

impl GridLayout {
    /// Creates a grid with `columns` columns and no gaps.
    ///
    /// `columns` is clamped to a minimum of `1`, so a zero column count still
    /// produces a valid single-column grid.
    #[must_use]
    pub fn new(columns: usize) -> Self {
        Self {
            columns: if columns == 0 { 1 } else { columns },
            column_gap: 0.0,
            row_gap: 0.0,
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

    /// The number of columns (always at least `1`).
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.columns
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

        // Content area available for the grid (may be unbounded on either axis).
        let content_width = (constraints.max.width - pad_x).max(0.0);
        let avail_height = (constraints.max.height - pad_y).max(0.0);

        let cols = self.columns.max(1);
        // usize → f32 for the column arithmetic (mirrors the count-to-f32
        // conversion in `FlexLayout`); column counts are tiny so no precision is
        // lost in practice.
        let cols_f = cols as f32;
        let gap_span = self.column_gap * (cols.saturating_sub(1) as f32);
        let cell_width = ((content_width - gap_span) / cols_f).max(0.0);

        let mut child_boxes = Vec::with_capacity(children.len());
        // Content-relative extents; start at the padding origin so an empty grid
        // sizes to its padding only.
        let mut max_right = padding.left;
        let mut y = 0.0_f32;

        let count = children.len();
        let mut index = 0;
        let mut first_row = true;
        while index < count {
            if !first_row {
                y += self.row_gap;
            }
            first_row = false;

            let remaining_height = (avail_height - y).max(0.0);
            let cell_constraints = BoxConstraints {
                min: Size::new(cell_width, 0.0),
                max: Size::new(cell_width, remaining_height),
            };

            let row_end = (index + cols).min(count);
            let mut row_height = 0.0_f32;
            for i in index..row_end {
                let col = i - index;
                let x = padding.left + (col as f32) * (cell_width + self.column_gap);
                let size = measurer.measure_child(i, cell_constraints);
                let origin = Point::new(x, padding.top + y);
                child_boxes.push(LayoutBox::new(Rect::new(origin, size)));
                row_height = row_height.max(size.height);
                max_right = max_right.max(x + size.width);
            }
            y += row_height;
            index = row_end;
        }

        let border_box = Size::new(max_right + padding.right, padding.top + y + padding.bottom);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::GridLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::LayoutStyle;
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
    }

    #[test]
    fn three_children_fill_two_columns_over_two_rows() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2), child(3)];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(100.0, 20.0),
            Size::new(100.0, 30.0),
            Size::new(100.0, 10.0),
        ]);
        // Content width 200, 2 columns, no gaps → cell width 100.
        let out = GridLayout::new(2).layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Row 0: two children side by side.
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size(), Size::new(100.0, 20.0));
        assert_eq!(b[1].origin(), Point::new(100.0, 0.0));
        assert_eq!(b[1].size(), Size::new(100.0, 30.0));
        // Row 1: below the tallest child of row 0 (30).
        assert_eq!(b[2].origin(), Point::new(0.0, 30.0));
        assert_eq!(b[2].size(), Size::new(100.0, 10.0));
        // Border box: full width 200, height = row0(30) + row1(10) = 40.
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
        // Content width 210, 2 columns, column gap 10 → cell = (210 - 10)/2 = 100.
        let grid = GridLayout::new(2).with_column_gap(10.0).with_row_gap(5.0);
        let out = grid.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(210.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Column 1 starts at cell(100) + column_gap(10) = 110.
        assert_eq!(b[1].origin(), Point::new(110.0, 0.0));
        // Row 1 starts at row0 height(20) + row_gap(5) = 25.
        assert_eq!(b[2].origin(), Point::new(0.0, 25.0));
        // Cells fixed to 100 wide even though children measure 0 wide.
        assert_eq!(b[0].size(), Size::new(100.0, 20.0));
    }

    #[test]
    fn zero_columns_clamps_to_one() {
        let grid = GridLayout::new(0);
        assert_eq!(grid.columns(), 1);
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 20.0), Size::new(50.0, 30.0)]);
        let out = grid.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(50.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Single column → children stack vertically.
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
        // Content width = 100 - 10 = 90; single column → cell 90.
        let out = GridLayout::new(1).layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(5.0, 5.0));
        // Border box: content 90 x 40 plus padding 10 on each axis.
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
