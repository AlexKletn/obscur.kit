//! `FlowLayout`: normal-flow (block) layout for the `Block` formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{Dimension, LayoutBox, LayoutStyle};

use super::{
    ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput, clamp_content_measured,
    intrinsic_inline_size,
};

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
/// * **Intrinsic width (CSS Sizing 3 §4.1)** — `min-content` / `max-content` /
///   `fit-content` are answered by *measuring* the children rather than by
///   arithmetic: the children are re-measured under a no-space (`min-content`) or
///   unbounded (`max-content`) inline offer, and the widest wins. See
///   [`intrinsic_inline_size`], which also explains why these cost an extra
///   measuring pass. In the **block** axis the two intrinsic sizes coincide with
///   the content height, so `height: max-content` is `height: auto` for a block
///   and needs no separate path.
/// * **Intrinsic min/max bounds** — `min-width` / `max-width` take the same
///   keywords, and are answered by the same probe as an intrinsic `width`, so
///   `min-width: min-content` floors the box at its widest unbreakable run
///   whatever its `width` resolved to. Their block-axis twins ride on the
///   coincidence above: an intrinsic `min-height` / `max-height` is the height the
///   children stacked to, at no extra measuring cost.
/// * **Child stacking** — each child is measured with
///   [`BoxConstraints::loose`] up to `(content_width − its inline margins,
///   remaining_height)`, placed at the content-box left edge **plus its
///   `margin-left`**, with its top at the running `y` offset.
/// * **Inline margins (CSS 2.1 §10.3.3)** — a child's *fixed* `margin-left` /
///   `margin-right` take from the inline space it is offered, so a `width: auto`
///   child fills what is left, and `margin-left` offsets it. An over-constrained
///   box (definite width + both margins) drops its *end* margin — the one on the
///   inline-end side of `style.direction`: `margin-right` under `ltr` (the child
///   stays line-left), `margin-left` under `rtl` (the child settles line-right).
/// * **`auto` inline margins (CSS 2.1 §10.3.3)** — an `auto` edge reserves
///   nothing; it *absorbs* the space left over once the child's used width is
///   known. Both edges `auto` splits the leftover evenly — `margin: 0 auto`
///   centring; one edge `auto` takes it all, pushing the box to the other side.
///   A `width: auto` child needs no special case: it already filled what it was
///   offered, so the leftover is zero and its `auto` margins resolve to zero,
///   exactly as §10.3.3 says. In the block axis `auto` is always zero
///   (§10.6.3): a container's height *is* its content, so there is no leftover
///   to absorb.
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
/// nuances and clearance. Floats, inline formatting / line boxes, and
/// writing-mode-aware block direction are likewise not modelled; block flow is
/// strictly vertical.
///
/// # Percentage margins resolve against the *width*, on every edge
///
/// A `%` margin — including `margin-top` / `margin-bottom` — resolves against this
/// container's `content_width` (CSS 2.1 §8.3). Reading that as a typo is the
/// natural reaction; it is deliberate. A block's height usually depends on its
/// content, so resolving a block-axis percentage against it would be circular. CSS
/// breaks the cycle by picking one axis, and the inline one is the axis that is
/// definite by the time children are laid out.
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

        // Available content height for measuring children under (may be infinite).
        let avail_content_height = (constraints.max.height - bp_y).max(0.0);

        // The content width a `width: auto` box takes: everything its container
        // offers. Also the `stretch` term `fit-content` is bounded by. It is
        // **unbounded** while this box is itself being measured for someone else's
        // `max-content`.
        let stretch = (constraints.max.width - bp_x).max(0.0);
        let explicit = style.width.resolve(constraints.max.width);
        // `auto` normally stretches to the space offered — but at the two ends of
        // the range there is no space to stretch to, and the offer is a QUESTION
        // about the content rather than an amount of room (see `BoxConstraints`'s
        // `constrain_allowing_overflow`, which reads the offers the same way).
        // CSS Sizing 3 §5.2 answers both: against an infinite available space
        // `auto` sizes to `max-content`, and under a min-content constraint it
        // sizes to `min-content`.
        //
        // * **Unbounded** — stretching would make the box infinitely wide and
        //   poison every size derived from it. This box is being measured for
        //   someone else's `max-content`.
        // * **Zero** — stretching to `0` makes the box vanish and its content
        //   overflow, and the number is worthless to whoever asked. This box is
        //   being measured for someone else's `min-content` — a `min-content`
        //   grid track, or a `min-width: min-content` ancestor. Without this an
        //   intrinsic probe answers `0` for every box that is not a bare text
        //   leaf, and the keyword is hollow one level down.
        //
        // Both apply to a percentage too, which §5.1 already turned into `auto` by
        // refusing to resolve against an indefinite basis (see `Dimension::resolve`)
        // — so `width: 50%` inside a `max-content` box contributes its content, not
        // half of infinity.
        //
        // # The case this gets wrong, deliberately
        //
        // A box in a *genuinely* zero-wide containing block (a 20px parent with
        // 10px of padding on each side) now reports its min-content and overflows,
        // where CSS says its used width is `0` and only its content overflows.
        // Zero cannot be both "no room" and "how little can you be?" in a model
        // that infers intent from the number — `BoxConstraints` would have to carry
        // the min-content constraint explicitly, a change to that type and to every
        // algorithm. Between the two readings this picks the one with a real caller:
        // the probe, which is dead without it. The degenerate box was overflowing
        // its parent either way.
        let width = if explicit.is_some() || style.width.is_intrinsic() {
            style.width
        } else if !stretch.is_finite() {
            Dimension::MaxContent
        } else if stretch <= 0.0 {
            Dimension::MinContent
        } else {
            style.width
        };
        // Content width: an intrinsic keyword is answered by *measuring* the
        // children (`min-content` / `max-content` / `fit-content`, spec §4.2); an
        // explicit width resolves against the max width; `auto` stretches. Never
        // negative. `box-sizing: border-box` subtracts the horizontal border +
        // padding from an explicit width (the `Auto` branch already does), keeping
        // the two consistent — and does not apply to an intrinsic width, which is
        // already a content width (see `intrinsic_inline_size`).
        let content_width =
            intrinsic_inline_size(width, children, stretch, avail_content_height, measurer)
                .unwrap_or_else(|| {
                    explicit
                        .map_or(stretch, |w| style.box_sizing.content_extent(w, bp_x))
                        .max(0.0)
                });
        // Clamp the content width into the authored min/max-width (spec §4.2). An
        // intrinsic BOUND is measured exactly like an intrinsic width — the same
        // probe, so `min-width: min-content` floors the box at the run it cannot
        // break, whatever the `width` said. `intrinsic_inline_size` answers `None`
        // for a bound that is not a keyword without measuring anything, so a
        // non-intrinsic bound costs nothing.
        let min_width_measured = intrinsic_inline_size(
            style.min_width,
            children,
            stretch,
            avail_content_height,
            measurer,
        );
        let max_width_measured = intrinsic_inline_size(
            style.max_width,
            children,
            stretch,
            avail_content_height,
            measurer,
        );
        let content_width = clamp_content_measured(
            content_width,
            (style.min_width, min_width_measured),
            (style.max_width, max_width_measured),
            constraints.max.width,
            bp_x,
            style.box_sizing,
        );

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
            // This child's top margin joins the currently-open collapsing set. A
            // block-axis `auto` margin is zero in normal flow (CSS 2.1 §10.6.3):
            // there is no leftover block space for it to absorb, because the
            // container's height is what the children add up to.
            //
            // `content_width` is the basis for a percentage margin on EVERY edge,
            // including this block-axis one: CSS 2.1 §8.3 resolves all four against
            // the containing block's *width*. `margin-top: 10%` of a 300px-wide
            // parent is 30px regardless of how tall anything is.
            pending = pending.adjoin(child_style.margin.top.resolve(content_width));

            // Separation before this child: the additive gap plus the collapsed
            // margin. For the first child this is just its (contained) top
            // margin, since `gap_acc` is 0.
            let separation = gap_acc + pending.solve();
            let remaining_height = (avail_content_height - (y + separation)).max(0.0);
            // Horizontal margins take from the child's available inline space
            // (CSS 2.1 §10.3.3: margin-left + border + padding + width + padding
            // + border + margin-right = the containing block's width), so a
            // `width: auto` child fills what is left rather than the whole box.
            // Only the *fixed* edges take: an `auto` margin reserves nothing, it
            // absorbs what is left after the child has been sized.
            let fixed_inline = child_style.margin.left.resolve(content_width)
                + child_style.margin.right.resolve(content_width);
            let child_width = (content_width - fixed_inline).max(0.0);
            let child_size = measurer.measure_child(
                index,
                BoxConstraints::loose(Size::new(child_width, remaining_height)),
            );
            // …and the leading margin offsets it — §10.3.3's inline solve, now that
            // the child's used width is known:
            //
            // * neither edge `auto` — `margin-left` offsets, and an over-constrained
            //   box (definite width plus both margins) drops its inline-*end*
            //   margin per the containing block's `direction`: `margin-right` under
            //   `ltr` (keep line-left), `margin-left` under `rtl` (keep line-right;
            //   the child's left edge moves inward by the whole leftover).
            // * both `auto` — the leftover splits evenly, which is `margin: 0 auto`
            //   centring.
            // * one `auto` — it takes the whole leftover, pushing the box to the
            //   other edge.
            //
            // A `width: auto` child needs no special case: it already filled the
            // space it was offered, so the leftover is zero and its `auto` margins
            // resolve to zero — exactly what §10.3.3 says. Centring engages only for
            // a child whose width is definite (or intrinsic, which is how
            // `width: fit-content; margin: 0 auto` centres).
            let leftover = (content_width - fixed_inline - child_size.width).max(0.0);
            let left_margin = match (
                child_style.margin.left.is_auto(),
                child_style.margin.right.is_auto(),
            ) {
                (true, true) => leftover / 2.0,
                (true, false) => leftover,
                (false, true) => child_style.margin.left.resolve(content_width),
                // Both inline margins are fixed. If the child's used width leaves
                // room (`leftover > 0`) the inline dimension is *over-constrained*,
                // and CSS drops the **end** margin "as determined by the writing
                // mode of the containing block" (css-writing-modes-4.txt:1988-1990).
                // Under `ltr` the end is line-right, so `margin-right` is dropped
                // and the child keeps `margin-left` — it stays against the
                // line-left edge (`leftover` is absorbed on the right, unchanged
                // behaviour). Under `rtl` the end is line-left, so `margin-left` is
                // the one dropped: the child keeps its `margin-right` against the
                // line-right edge, which shifts its left edge inward by the whole
                // `leftover`. A `width: auto` child has already filled its offer, so
                // `leftover` is `0` and this is inert whichever the direction.
                (false, false) => {
                    let margin_left = child_style.margin.left.resolve(content_width);
                    if style.direction.is_rtl() {
                        margin_left + leftover
                    } else {
                        margin_left
                    }
                }
            };
            let left = bp_left + left_margin;

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
                pending = pending.adjoin(child_style.margin.bottom.resolve(content_width));
                // Place the zero-height box at the (current) collapsed boundary;
                // it does not advance `y` and does not close the set.
                let top = bp_top + y + gap_acc + pending.solve();
                child_boxes.push(LayoutBox::new(Rect::new(Point::new(left, top), child_size)));
                continue;
            }

            // A real child closes the collapsing set: it sits below the
            // collapsed separation, then a fresh set opens with its bottom
            // margin and the gap accumulator resets.
            let top = y + separation;
            child_boxes.push(LayoutBox::new(Rect::new(
                Point::new(left, bp_top + top),
                child_size,
            )));
            y = top + child_size.height;
            pending =
                CollapsingMargins::EMPTY.adjoin(child_style.margin.bottom.resolve(content_width));
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
        // Clamp the content height into the authored min/max-height (spec §4.2) —
        // e.g. `min-height: 100vh` fills a tall viewport even when the content is
        // short. Previously dropped: flow honoured only the parent constraints.
        //
        // An intrinsic bound needs no probe in the BLOCK axis: a block does not
        // reflow vertically, so `min-content`, `max-content` and `fit-content` all
        // coincide with the height the children already stacked to (the same
        // reason `height: max-content` is `height: auto` here — see the type docs).
        let block_intrinsic =
            |bound: Dimension| bound.is_intrinsic().then_some(children_content_height);
        let content_height = clamp_content_measured(
            content_height,
            (style.min_height, block_intrinsic(style.min_height)),
            (style.max_height, block_intrinsic(style.max_height)),
            constraints.max.height,
            bp_y,
            style.box_sizing,
        );

        // The box reports the size it resolved to. A *tight* constraint still
        // dictates (an absolutely-positioned box with both insets is assigned its
        // size, not offered it); a *loose* one only offers, so content that does
        // not fit overflows rather than reporting the space it was given.
        //
        // `min-content` and `fit-content` are what make that distinction visible —
        // both are *defined* to overflow rather than drop below the widest
        // unbreakable run (CSS 2.1 §10.3.5) — but it was equally true of an
        // explicit `width: 500px` in a 40px container, which used to report 40.
        // Anything that fits is unaffected.
        let border_box = Size::new(content_width + bp_x, content_height + bp_y);
        let size = constraints.constrain_allowing_overflow(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::FlowLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{Dimension, LayoutStyle, MarginEdges};
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
    }

    /// A child carrying only block-axis (top/bottom) margins.
    fn child_margins(raw: u64, top: f32, bottom: f32) -> LayoutChild {
        let style = LayoutStyle {
            margin: MarginEdges::px(top, 0.0, bottom, 0.0),
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
    fn min_height_floors_short_content() {
        // Content is 20 tall, but `min-height: 200` floors the box to 200 — the
        // §4.2 clamp flow used to drop (it honoured only parent constraints).
        let style = LayoutStyle {
            min_height: Dimension::Px(200.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(100.0, 20.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().height, 200.0);
    }

    #[test]
    fn max_width_caps_content_width() {
        // Available width is 100, but `max-width: 60` caps the box.
        let style = LayoutStyle {
            max_width: Dimension::Px(60.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(100.0, 20.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size().width, 60.0);
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

    // ---- inline margins (CSS 2.1 §10.3.3) ----------------------------------

    /// A child carrying only inline-axis (left/right) margins.
    fn child_inline_margins(raw: u64, left: f32, right: f32) -> LayoutChild {
        LayoutChild::new(
            NodeId::from_raw(raw),
            LayoutStyle {
                margin: MarginEdges::px(0.0, right, 0.0, left),
                ..LayoutStyle::default()
            },
        )
    }

    #[test]
    fn a_left_margin_offsets_the_child() {
        // Flow used to pin every child to the content-box left edge, so
        // `margin-left` did nothing at all — a documented deferral, now closed.
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(50.0, 20.0)]);
        let out = FlowLayout.layout(
            &LayoutStyle::default(),
            &[child_inline_margins(1, 40.0, 0.0)],
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(40.0, 0.0));
    }

    #[test]
    fn inline_margins_take_from_the_space_the_child_is_offered() {
        // §10.3.3: margin-left + width + margin-right fills the containing block,
        // so an `auto`-width child gets what the margins leave — not the whole box.
        // A child wider than the box is clamped to what it was OFFERED, which is
        // what makes the offered width readable from the placed box.
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(1000.0, 20.0)]);
        let out = FlowLayout.layout(
            &LayoutStyle::default(),
            &[child_inline_margins(1, 30.0, 20.0)],
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        let placed = out.child_boxes()[0];
        assert_eq!(placed.origin().x, 30.0, "offset by the leading margin");
        assert_eq!(
            placed.size().width,
            250.0,
            "300 − 30 − 20: both margins take from the inline space"
        );
    }

    #[test]
    fn inline_margins_never_yield_a_negative_offered_width() {
        // Margins wider than the box would otherwise offer a negative width.
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(1000.0, 20.0)]);
        let out = FlowLayout.layout(
            &LayoutStyle::default(),
            &[child_inline_margins(1, 400.0, 400.0)],
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].size().width, 0.0);
    }

    #[test]
    fn a_left_margin_stacks_with_the_containers_padding() {
        // The child sits at padding + margin, not at one or the other.
        let style = LayoutStyle {
            padding: Insets::uniform(10.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(50.0, 20.0)]);
        let out = FlowLayout.layout(
            &style,
            &[child_inline_margins(1, 40.0, 0.0)],
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(50.0, 10.0));
    }

    #[test]
    fn a_child_with_no_inline_margins_is_placed_exactly_as_before() {
        // The change must be inert for the overwhelming majority of boxes.
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(100.0, 20.0)]);
        let out = FlowLayout.layout(
            &LayoutStyle::default(),
            &[child(1)],
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(0.0, 0.0));
    }

    // ---- `auto` at the two ends of the offer (CSS Sizing 3 §5.2) -----------

    /// A child that answers the offer: it fills the inline space it is given, but
    /// never below `min` (its longest unbreakable word) and never above `max` (its
    /// whole content on one line).
    struct Paragraph {
        min: f32,
        max: f32,
    }

    impl crate::algorithm::ChildMeasurer for Paragraph {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            Size::new(constraints.max.width.clamp(self.min, self.max), 10.0)
        }
    }

    fn paragraph_box_width(offer: f32) -> f32 {
        FlowLayout
            .layout(
                &LayoutStyle::default(),
                &[child(1)],
                BoxConstraints::loose(Size::new(offer, 100.0)),
                &mut Paragraph {
                    min: 30.0,
                    max: 200.0,
                },
            )
            .size()
            .width
    }

    #[test]
    fn an_auto_width_box_offered_no_space_reports_its_min_content() {
        // A zero offer is the min-content probe, so the box answers with the 30 it
        // cannot break below rather than stretching to `0` and vanishing. This is
        // what carries an intrinsic probe THROUGH a box: without it a `min-content`
        // grid track or a `min-width: min-content` ancestor gets `0` for every item
        // that is not a bare text leaf.
        assert_eq!(paragraph_box_width(0.0), 30.0);
    }

    #[test]
    fn an_auto_width_box_offered_unbounded_space_reports_its_max_content() {
        // The other end, unchanged: an infinite offer yields a finite want.
        assert_eq!(paragraph_box_width(f32::INFINITY), 200.0);
    }

    #[test]
    fn an_auto_width_box_offered_real_space_still_stretches_into_it() {
        // The guard that matters most: between the two ends `auto` fills its
        // container exactly as it always has.
        assert_eq!(paragraph_box_width(150.0), 150.0);
        assert_eq!(paragraph_box_width(300.0), 300.0);
    }
}
