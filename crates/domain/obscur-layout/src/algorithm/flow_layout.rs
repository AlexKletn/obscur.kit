//! `FlowLayout`: normal-flow (block) layout for the `Block` formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{LayoutBox, LayoutStyle};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// Normal-flow / block layout (spec §4.1, §4.2).
///
/// Stacks children top-to-bottom in document order, each occupying the full
/// content width, and sizes the node to fit unless an explicit height is given.
///
/// # Rules
///
/// * **Content width** — resolves `style.width` against the constraint's max
///   width; if `Auto`, uses the available content width (`constraints.max.width`
///   minus horizontal border + padding). Clamped to be non-negative.
/// * **Child stacking** — each child is measured with
///   [`BoxConstraints::loose`] up to `(content_width, remaining_height)`, placed
///   at the content-box left edge, with its top at the running `y` offset.
/// * **Block margins + vertical margin collapsing (CSS 2.1 §8.3.1)** — each
///   in-flow child's block-axis (`margin.top` / `margin.bottom`) margins take
///   effect, and the bottom margin of one block collapses with the top margin
///   of the next into a single used margin. The used value of a collapsing set
///   is the **`{max_positive, min_negative}`** reduction: adjoin a margin by
///   keeping the largest positive and the most-negative seen; solve the set as
///   `max_positive + min_negative` (so two positives take the larger, two
///   negatives take the more-negative, and a mixed pair sums — `+30` with `−20`
///   → `+10`). The separation placed between two siblings is
///   `gap + collapsed_margin`: `row-gap` does **not** collapse and is additive
///   (Box Alignment), so with zero child margins the separation is exactly
///   `gap` — byte-identical to a gap-only stack. An empty, zero-height child
///   with no top/bottom border or padding has its own top and bottom margins
///   *adjoining*, so it **collapses through**: the open collapsing set spans
///   across it and a run of such empties between two siblings collapses into a
///   single margin.
/// * **Content height** — resolves `style.height` if set, otherwise the last
///   child's collapsed bottom edge (the trailing collapsing set is contained,
///   see below). Clamped to be non-negative.
/// * **Border box** — content size plus border and padding, then clamped into
///   the incoming `constraints`.
///
/// Child boxes are positioned relative to this node's origin, already offset into
/// the content box by border + padding.
///
/// # Deferred — the remaining §8.3.1 piece
///
/// **Parent–child / root collapse-through across the container boundary** (a
/// first child's top margin or a last child's bottom margin protruding *out* of
/// this box to collapse with the parent's own margin) is **not** modelled: it
/// requires the [`LayoutOutput`] / [`LayoutAlgorithm`] contract to carry a
/// block's *adjoining top/bottom margins* up to the parent formatting context —
/// a protocol change touching every algorithm. For now the first child's top
/// margin and the last child's bottom margin are **contained** within this box's
/// content (no protrusion). Also out of scope: BFC-root collapse suppression
/// nuances, clearance, and **horizontal margins** — children stay pinned to the
/// content-box left edge, so `margin-left` / `margin-right` (including
/// `margin: 0 auto` centering) are not applied. Floats, inline formatting /
/// line boxes, and writing-mode-aware block direction are likewise not modelled;
/// block flow is strictly vertical.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlowLayout;

/// A set of vertical margins that are *adjoining* and therefore collapse into a
/// single used margin (CSS 2.1 §8.3.1).
///
/// The set is summarised by two running extremes — the largest positive margin
/// and the most-negative margin seen — so that:
///
/// * two positive margins collapse to the **larger** (positives don't add),
/// * two negative margins collapse to the **most-negative** (negatives don't
///   add), and
/// * a mixed set collapses to their **sum** (`max_positive + min_negative`).
///
/// This is the canonical `{max_positive, min_negative}` reduction; folding each
/// margin in with [`adjoin`](Self::adjoin) and reading [`solve`](Self::solve)
/// yields the used margin for the whole adjoining run regardless of order.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CollapsingMargins {
    /// The largest non-negative margin in the set (`0` if none).
    max_positive: f32,
    /// The most-negative margin in the set (`0` if none).
    min_negative: f32,
}

impl CollapsingMargins {
    /// The empty set — no margins have collapsed in yet (used margin `0`).
    const EMPTY: Self = Self {
        max_positive: 0.0,
        min_negative: 0.0,
    };

    /// Folds one margin into the set, keeping the running positive/negative
    /// extremes. Order-independent, so an entire adjoining run reduces to the
    /// same result however it is accumulated.
    #[must_use]
    fn adjoin(self, margin: f32) -> Self {
        if margin >= 0.0 {
            Self {
                max_positive: self.max_positive.max(margin),
                ..self
            }
        } else {
            Self {
                min_negative: self.min_negative.min(margin),
                ..self
            }
        }
    }

    /// The used margin for the collapsed set: `max_positive + min_negative`.
    #[must_use]
    fn solve(self) -> f32 {
        self.max_positive + self.min_negative
    }
}

impl LayoutAlgorithm for FlowLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let border = style.border;
        let padding = style.padding;
        let bp_left = border.left + padding.left;
        let bp_top = border.top + padding.top;
        let bp_x = bp_left + border.right + padding.right;
        let bp_y = bp_top + border.bottom + padding.bottom;

        // Content width: explicit width (resolved against the max width) else the
        // available content width. Never negative. `box-sizing: border-box`
        // subtracts the horizontal border + padding from an explicit width (the
        // `Auto` branch already does), keeping the two consistent.
        let content_width = style
            .width
            .resolve(constraints.max.width)
            .map_or(constraints.max.width - bp_x, |w| {
                style.box_sizing.content_extent(w, bp_x)
            })
            .max(0.0);

        // Available content height for measuring children under (may be infinite).
        let avail_content_height = (constraints.max.height - bp_y).max(0.0);

        let mut child_boxes = Vec::with_capacity(children.len());
        // `y` is the collapsed bottom edge of the last placed *real* (non
        // collapse-through) child, in content-box coordinates; `pending` is the
        // open collapsing set of adjoining margins at the current boundary
        // (CSS 2.1 §8.3.1); `gap_acc` is the non-collapsing `row-gap` accrued
        // across that boundary (additive per Box Alignment, never collapses).
        let mut y = 0.0_f32;
        let mut pending = CollapsingMargins::EMPTY;
        let mut gap_acc = 0.0_f32;
        for (index, child) in children.iter().enumerate() {
            let child_style = child.style();
            if index > 0 {
                gap_acc += style.gap;
            }
            // This child's top margin joins the currently-open collapsing set.
            pending = pending.adjoin(child_style.margin.top);

            // Separation before this child: the additive gap plus the collapsed
            // margin. For the first child this is just its (contained) top
            // margin, since `gap_acc` is 0.
            let separation = gap_acc + pending.solve();
            let remaining_height = (avail_content_height - (y + separation)).max(0.0);
            let child_size = measurer.measure_child(
                index,
                BoxConstraints::loose(Size::new(content_width, remaining_height)),
            );

            // An empty, zero-height child with no top/bottom border or padding
            // has adjoining top & bottom margins: it collapses *through*. The
            // open set spans across it — its bottom margin folds in and neither
            // `y` nor the set is closed — so a run of empties yields one margin.
            let collapses_through = child_size.height == 0.0
                && child_style.border.top == 0.0
                && child_style.border.bottom == 0.0
                && child_style.padding.top == 0.0
                && child_style.padding.bottom == 0.0;

            if collapses_through {
                pending = pending.adjoin(child_style.margin.bottom);
                // Place the zero-height box at the (current) collapsed boundary;
                // it does not advance `y` and does not close the set.
                let top = bp_top + y + gap_acc + pending.solve();
                child_boxes.push(LayoutBox::new(Rect::new(
                    Point::new(bp_left, top),
                    child_size,
                )));
                continue;
            }

            // A real child closes the collapsing set: it sits below the
            // collapsed separation, then a fresh set opens with its bottom
            // margin and the gap accumulator resets.
            let top = y + separation;
            child_boxes.push(LayoutBox::new(Rect::new(
                Point::new(bp_left, bp_top + top),
                child_size,
            )));
            y = top + child_size.height;
            pending = CollapsingMargins::EMPTY.adjoin(child_style.margin.bottom);
            gap_acc = 0.0;
        }

        // Auto content height: the last real child's collapsed bottom edge plus
        // the trailing collapsing set (the last child's bottom margin — and any
        // trailing collapse-through empties — are *contained* within this box,
        // not protruded; see the module docs). With zero child margins this is
        // exactly `y`, byte-identical to a gap-only stack.
        let children_content_height = y + pending.solve();

        // Content height: explicit height (mapped through `box-sizing`) else the
        // accumulated children height. `border-box` subtracts the vertical border
        // + padding so the box ends up at its authored size.
        let content_height = style
            .height
            .resolve(constraints.max.height)
            .map_or(children_content_height, |h| {
                style.box_sizing.content_extent(h, bp_y)
            })
            .max(0.0);

        let border_box = Size::new(content_width + bp_x, content_height + bp_y);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::FlowLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{Dimension, LayoutStyle};
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
    }

    /// A child carrying only block-axis (top/bottom) margins.
    fn child_margins(raw: u64, top: f32, bottom: f32) -> LayoutChild {
        let style = LayoutStyle {
            margin: Insets {
                top,
                bottom,
                left: 0.0,
                right: 0.0,
            },
            ..LayoutStyle::default()
        };
        LayoutChild::new(NodeId::from_raw(raw), style)
    }

    #[test]
    fn stacks_two_children_and_sizes_to_fit() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(100.0, 20.0), Size::new(100.0, 30.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );

        let boxes = out.child_boxes();
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[0].size(), Size::new(100.0, 20.0));
        // Second child sits directly below the first (no gap).
        assert_eq!(boxes[1].origin(), Point::new(0.0, 20.0));
        assert_eq!(boxes[1].size(), Size::new(100.0, 30.0));
        // Node sizes to fit the stacked children.
        assert_eq!(out.size(), Size::new(100.0, 50.0));
    }

    #[test]
    fn gap_separates_children() {
        let style = LayoutStyle {
            gap: 8.0,
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(100.0, 20.0), Size::new(100.0, 30.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );

        let boxes = out.child_boxes();
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[1].origin(), Point::new(0.0, 28.0));
        // 20 + 8 gap + 30 = 58.
        assert_eq!(out.size(), Size::new(100.0, 58.0));
    }

    #[test]
    fn fixed_height_overrides_content_height() {
        let style = LayoutStyle::default().with_height(Dimension::Px(200.0));
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(100.0, 20.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size(), Size::new(100.0, 200.0));
    }

    #[test]
    fn border_and_padding_offset_children_and_grow_box() {
        let style = LayoutStyle {
            padding: Insets::uniform(4.0),
            border: Insets::uniform(1.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 10.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        // Content origin offset by border(1) + padding(4) = 5 on each axis.
        assert_eq!(out.child_boxes()[0].origin(), Point::new(5.0, 5.0));
        // Content width = 100 - 10 = 90; border box = 90 + 10 = 100 wide.
        // Content height = 10; border box = 10 + 10 = 20 tall.
        assert_eq!(out.size(), Size::new(100.0, 20.0));
    }

    #[test]
    fn empty_children_size_to_insets_only() {
        let style = LayoutStyle {
            padding: Insets::uniform(3.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::default();
        let out = FlowLayout.layout(
            &style,
            &[],
            BoxConstraints::loose(Size::new(50.0, 50.0)),
            &mut measurer,
        );
        assert!(out.is_empty());
        // Content width = 50 - 6 = 44, height = 0; border box = 50 x 6.
        assert_eq!(out.size(), Size::new(50.0, 6.0));
    }

    // ----- vertical margin collapsing (CSS 2.1 §8.3.1) --------------------

    /// The `{max_positive, min_negative}` accumulator: positives take the max,
    /// negatives the min, a mixed set sums.
    #[test]
    fn collapsing_margins_accumulator_solves() {
        use super::CollapsingMargins;
        // Two positives → the larger.
        assert_eq!(
            CollapsingMargins::EMPTY.adjoin(20.0).adjoin(30.0).solve(),
            30.0
        );
        // Two negatives → the most negative.
        assert_eq!(
            CollapsingMargins::EMPTY.adjoin(-4.0).adjoin(-9.0).solve(),
            -9.0
        );
        // Mixed → the sum of the extremes.
        assert_eq!(
            CollapsingMargins::EMPTY.adjoin(20.0).adjoin(-8.0).solve(),
            12.0
        );
    }

    /// Adjacent siblings: bottom(20) collapses with top(30) → one 30px gap, NOT
    /// 50. B's top edge sits at A's bottom + 30.
    #[test]
    fn adjacent_margins_collapse_to_max() {
        let style = LayoutStyle::default(); // gap: 0
        let children = [child_margins(1, 0.0, 20.0), child_margins(2, 30.0, 0.0)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(100.0, 40.0), Size::new(100.0, 50.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let boxes = out.child_boxes();
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        // A bottom = 40; collapsed margin = max(20, 30) = 30; B top = 70.
        assert_eq!(boxes[1].origin(), Point::new(0.0, 70.0));
        // Height = 70 + 50 = 120 (no last-child bottom margin here).
        assert_eq!(out.size(), Size::new(100.0, 120.0));
    }

    /// A mixed adjacent pair sums: bottom(20) with top(−8) → 12px separation.
    #[test]
    fn adjacent_margins_mixed_sum() {
        let style = LayoutStyle::default();
        let children = [child_margins(1, 0.0, 20.0), child_margins(2, -8.0, 0.0)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(100.0, 40.0), Size::new(100.0, 50.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        // A bottom = 40; collapsed = 20 + (−8) = 12; B top = 52.
        assert_eq!(out.child_boxes()[1].origin(), Point::new(0.0, 52.0));
    }

    /// The non-collapsing `gap` is additive on top of the collapsed margin.
    #[test]
    fn gap_is_additive_over_collapsed_margin() {
        let style = LayoutStyle {
            gap: 10.0,
            ..LayoutStyle::default()
        };
        let children = [child_margins(1, 0.0, 20.0), child_margins(2, 30.0, 0.0)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(100.0, 40.0), Size::new(100.0, 50.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        // Separation = gap(10) + collapsed(30) = 40; B top = 40 + 40 = 80.
        assert_eq!(out.child_boxes()[1].origin(), Point::new(0.0, 80.0));
    }

    /// An empty, zero-height, border/padding-free block collapses through: its
    /// `margin: 15 0` joins the run so A→B is ONE 15px gap, not two.
    #[test]
    fn empty_block_collapses_through() {
        let style = LayoutStyle::default(); // gap: 0
        let children = [
            child_margins(1, 0.0, 0.0),
            child_margins(2, 15.0, 15.0), // the empty spacer (height 0)
            child_margins(3, 0.0, 0.0),
        ];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(100.0, 40.0),
            Size::new(100.0, 0.0),
            Size::new(100.0, 50.0),
        ]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let boxes = out.child_boxes();
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        // The run 0 (A.bottom) ∪ 15 (E.top) ∪ 15 (E.bottom) ∪ 0 (B.top) → 15.
        // B sits at A.bottom(40) + 15 = 55, a single collapsed gap.
        assert_eq!(boxes[2].origin(), Point::new(0.0, 55.0));
        assert_eq!(out.size(), Size::new(100.0, 105.0));
    }

    /// The first child's top margin is contained inside the content box (no
    /// protrusion), and the last child's bottom margin extends the content box.
    #[test]
    fn first_top_and_last_bottom_margins_are_contained() {
        let style = LayoutStyle::default();
        let children = [child_margins(1, 12.0, 7.0)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(100.0, 30.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        // Top margin 12 pushes the child down inside the box.
        assert_eq!(out.child_boxes()[0].origin(), Point::new(0.0, 12.0));
        // Height = 12 (top) + 30 (child) + 7 (bottom, contained) = 49.
        assert_eq!(out.size(), Size::new(100.0, 49.0));
    }

    /// Two negative adjacent margins take the most-negative, pulling B upward.
    #[test]
    fn negative_margins_pull_sibling_up() {
        let style = LayoutStyle::default();
        let children = [child_margins(1, 0.0, -6.0), child_margins(2, -10.0, 0.0)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(100.0, 40.0), Size::new(100.0, 50.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        // Collapsed = min(−6, −10) = −10; B top = 40 + (−10) = 30 (overlaps A).
        assert_eq!(out.child_boxes()[1].origin(), Point::new(0.0, 30.0));
    }
}
