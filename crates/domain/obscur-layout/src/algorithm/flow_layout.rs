//! `FlowLayout`: normal-flow (block) layout for the `Block` formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{Dimension, LayoutBox, LayoutStyle, WritingMode};

use super::{
    ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput, clamp_content,
    clamp_content_measured, intrinsic_inline_size,
};

/// Normal-flow / block layout (spec §4.1, §4.2).
///
/// Stacks children top-to-bottom in document order, each occupying the full
/// content width, and sizes the node to fit unless an explicit height is given.
///
/// # Rules
///
/// * **Content width** — resolves `style.width` against the containing block the
///   caller stated ([`BoxConstraints::percent_width`], which falls back to the
///   constraint's max width); if `Auto`, uses the available content width
///   (`constraints.max.width` minus horizontal border + padding). Clamped to be
///   non-negative.
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
///   `margin-left`**, with its top at the running `y` offset. When this box's own
///   block size is definite, that offer carries this box's **content box** as the
///   child's percentage basis — a separate quantity from the offer, see below.
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
/// * **Content height** — resolves `style.height` if set — against the containing
///   block ([`BoxConstraints::percent_height`]), not against the space this box was
///   offered — otherwise the last child's collapsed bottom edge (the trailing
///   collapsing set is contained, see below). Clamped to be non-negative.
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
/// nuances and clearance. Floats and inline formatting / line boxes are likewise
/// not modelled. A **vertical** `writing-mode` rotates the block axis to
/// horizontal — that path is [`layout_vertical`], and the rules above describe the
/// default `horizontal-tb` mode.
///
/// # Percentage margins resolve against the *width*, on every edge
///
/// A `%` margin — including `margin-top` / `margin-bottom` — resolves against this
/// container's `content_width` (CSS 2.1 §8.3). Reading that as a typo is the
/// natural reaction; it is deliberate. A block's height usually depends on its
/// content, so resolving a block-axis percentage against it would be circular. CSS
/// breaks the cycle by picking one axis, and the inline one is the axis that is
/// definite by the time children are laid out.
///
/// # A percentage *size* resolves against the containing block, not the space left
///
/// The offer a child is measured under and the basis its percentages resolve
/// against are two different quantities, and this algorithm used to have only one.
/// Each child was offered the block extent *remaining* below its predecessors and
/// resolved `height: 50%` against that offer, so in a 240px-tall container the
/// first child was right (120) and every sibling after it compounded the error —
/// the second `50%` took half of the 120 that remained (60), the second `30%` of
/// two came back 50, `20%`/`80%` came back 48/154, and a second `height: 100%`
/// computed to **zero** and disappeared with nothing to say why.
///
/// css-sizing-3.txt:1027 is unambiguous, and it also settles what should happen
/// instead of the clamping:
///
/// > Otherwise, the percentage is resolved against the containing block's size.
/// > (The containing block's size is not re-resolved based on the resulting size of
/// > the box; the contents might thus **overflow** or underflow the containing
/// > block.)
///
/// So the remaining extent keeps its job — the room left for the next sibling, and
/// the space an `auto`/stretch size fills — while the containing block's content box
/// travels alongside it as [`BoxConstraints::percent_basis`]. Two children at
/// `height: 100%` are now 240 each and the second one starts at the container's
/// bottom edge; the container stays 240 and the overflow is the specified outcome,
/// not a defect to clamp away.
///
/// The basis is attached only when this box's block size is **definite**. Against an
/// indefinite one a percentage should compute to `auto`; it instead still resolves
/// against the space available, which is the pre-existing (and separately recorded)
/// gap — that behaviour is deliberately left exactly as it was rather than half-changed
/// here.
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
        // A vertical writing mode rotates the block axis to horizontal; its
        // placement is a separate path so this — the default `horizontal-tb` — is
        // left byte-for-byte unchanged (css-writing-modes-4 §6.2).
        if style.writing_mode.is_vertical() {
            return layout_vertical(style, children, constraints, measurer);
        }
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
        // See `FlexLayout`: a percentage resolves against the containing block the
        // caller states, `max` is only the available space for an `auto` size.
        let explicit = style.width.resolve(constraints.percent_width());
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
        // non-intrinsic bound costs nothing. A *percentage* bound resolves against
        // the containing block, the same basis `explicit` used above — `max.width` is
        // the space offered, and the two coincide when no caller stated a basis.
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
            constraints.percent_width(),
            bp_x,
            style.box_sizing,
        );

        // This box's **definite** content block size, if it has one: the basis a
        // child's percentage block size resolves against (css-sizing-3.txt:1027 —
        // "the percentage is resolved against the containing block's size").
        // Resolved before the children are touched, because that is the whole
        // point: the containing block's size does not depend on them.
        //
        // The min/max clamp is applied here too, so the basis is the *used* content
        // height rather than the authored one (`height: 50%; max-height: 100px`
        // gives descendants 100, not 120). An intrinsic bound cannot be measured
        // yet — it is the height the children stack to — so it imposes no limit on
        // the basis; the same clamp runs again below with the intrinsic bounds
        // measured, and re-applying an already-applied numeric bound is inert.
        //
        // [`None`] when `height` is `auto` (or a percentage against an indefinite
        // basis): the block size is **indefinite**, and this deliberately does not
        // try to invent one. That is the separate known gap — a percentage against
        // an indefinite basis should compute to `auto` and here still resolves
        // against the space available.
        let definite_content_height = style.height.resolve(constraints.percent_height()).map(|h| {
            let content = style.box_sizing.content_extent(h, bp_y).max(0.0);
            clamp_content(
                content,
                style.min_height,
                style.max_height,
                constraints.percent_height(),
                bp_y,
                style.box_sizing,
            )
        });
        // The percentage basis handed to every child: this box's content box.
        //
        // # Why this is not `remaining_height`
        //
        // Flow used to offer each child the block space *left over* below its
        // predecessors and let the child resolve its percentage against that offer,
        // so the first child of a 240px-tall box was right (`50%` → 120) and every
        // subsequent sibling compounded the error: the second `50%` read 50% of the
        // 120 that remained (→ 60), a second `height: 100%` computed to zero and
        // vanished. The two quantities have to be separate, because they answer
        // different questions: the remaining extent is how much room is left (for
        // placing the next sibling and for `auto`/stretch sizing, unchanged below),
        // while the percentage basis is the containing block's size — which
        // css-sizing-3.txt:1027 is explicit is *not* re-resolved from what the
        // earlier boxes used, "the contents might thus overflow or underflow the
        // containing block".
        //
        // [`BoxConstraints::percent_basis`] is a single [`Size`], so it is attached
        // only when the block axis is definite. Leaving it unset keeps an
        // indefinite containing block indefinite (percentages fall back to
        // resolving against `max`, exactly as before). The inline component is the
        // containing block's content width either way — the offer differs from it
        // only by the child's own inline margins, which a percentage does not
        // subtract (CSS 2.1 §10.2).
        let percent_basis = definite_content_height.map(|h| Size::new(content_width, h));

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
            // The offer is the space available; the basis (when this box has one) is
            // what a percentage resolves against. See `percent_basis` above for why
            // conflating the two made every sibling after the first wrong.
            let child_constraints = BoxConstraints::loose(Size::new(child_width, remaining_height));
            let child_size = measurer.measure_child(
                index,
                percent_basis.map_or(child_constraints, |basis| {
                    child_constraints.with_percent_basis(basis)
                }),
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

        // Content height: the definite height resolved above (already mapped through
        // `box-sizing`, which under `border-box` subtracts the vertical border +
        // padding so the box ends up at its authored size) else the accumulated
        // children height.
        //
        // It is resolved against [`BoxConstraints::percent_height`] — the containing
        // block the caller stated — and no longer against `max.height`, which is the
        // space this box was *offered*. The two coincide whenever no caller stated a
        // basis, so this is inert for every existing caller and correct for the new
        // one: a `height: 50%` block nested in a `height: 50%` block now halves its
        // containing block instead of halving whatever room its siblings left.
        let content_height = definite_content_height
            .unwrap_or(children_content_height)
            .max(0.0);
        // Clamp the content height into the authored min/max-height (spec §4.2) —
        // e.g. `min-height: 100vh` fills a tall viewport even when the content is
        // short. Previously dropped: flow honoured only the parent constraints.
        //
        // An intrinsic bound needs no probe in the BLOCK axis: a block does not
        // reflow vertically, so `min-content`, `max-content` and `fit-content` all
        // coincide with the height the children already stacked to (the same
        // reason `height: max-content` is `height: auto` here — see the type docs).
        //
        // A *percentage* bound resolves against the containing block, like the
        // `height` above and for the same reason: `min-height: 100%` on the second
        // of two siblings used to read the room the first left over.
        let block_intrinsic =
            |bound: Dimension| bound.is_intrinsic().then_some(children_content_height);
        let content_height = clamp_content_measured(
            content_height,
            (style.min_height, block_intrinsic(style.min_height)),
            (style.max_height, block_intrinsic(style.max_height)),
            constraints.percent_height(),
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

/// A child placed in the block-axis coordinate system of a vertical writing mode,
/// before it is mapped to a physical origin.
struct VerticalPlacement {
    /// Distance from the container's block-start content edge to this child's own
    /// block-start edge, along the (horizontal) block axis.
    block_start_offset: f32,
    /// Distance from the container's inline-start content edge (the top) to this
    /// child's top edge, along the (vertical) inline axis.
    inline_offset: f32,
    /// The child's physical size (width along the block axis, height along inline).
    size: Size,
}

/// Normal-flow block layout for a **vertical** writing mode (css-writing-modes-4
/// §6.2): the block axis is horizontal, so in-flow children stack left/right
/// instead of top/bottom.
///
/// The block axis is physical **width**; the inline axis is physical **height**.
/// `vertical-lr` progresses left-to-right from the left content edge;
/// `vertical-rl` progresses right-to-left from the right content edge. Children's
/// logical block-start / block-end margins are read from the physical edges the
/// mode maps them to (already folded from `margin-block-*` upstream by the style
/// crate): for `vertical-rl` block-start is the right edge (`margin.right`), for
/// `vertical-lr` it is the left edge (`margin.left`). Block-axis margin collapsing
/// (CSS 2.1 §8.3.1) and the non-collapsing `gap` work exactly as in the horizontal
/// path, only along the rotated axis.
///
/// The inline (vertical) axis carries each child's used height and its top/bottom
/// margins, resolved as the §10.3.3 inline solve on the vertical axis.
///
/// Percentage *sizes* resolve against the containing block on both axes, exactly as
/// in the horizontal path and for the same reason — see [`FlowLayout`]'s note on why
/// that is not the remaining extent. Here the axis that used to compound across
/// siblings is the physical **width**: two `width: 50%` children of a 480-wide
/// `vertical-lr` container are 240 each, where the second used to read half of the
/// 240 that remained.
///
/// # Modelled to the point the block-progression is correct
///
/// Intrinsic inline sizing along the vertical axis (`height: max-content` on a
/// vertical container) is not probed — a vertical `auto` inline size stretches to
/// the offer, and an intrinsic min/max-height bound degrades to `auto` — mirroring
/// the deliberate one-axis limit of [`intrinsic_inline_size`]. The block axis
/// (width) shrink-wraps its children when `width: auto`.
fn layout_vertical(
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

    let rl = matches!(style.writing_mode, WritingMode::VerticalRl);

    // Inline extent = physical height: an explicit height (through `box-sizing`)
    // else a stretch to the inline space offered (the vertical analogue of a
    // `width: auto` block stretching), clamped into min/max-height. Intrinsic
    // bounds degrade to `auto` (no vertical-axis probe).
    //
    // A percentage `height` — and a percentage min/max-height — resolves against
    // [`BoxConstraints::percent_height`], the containing block the caller stated,
    // not against the space this box was offered; the two coincide when no caller
    // stated a basis. This is the vertical mirror of the horizontal path's block
    // axis, and matters for the same reason: a vertical container's children are
    // measured under a *shrinking* block offer.
    let inline_avail = (constraints.max.height - bp_y).max(0.0);
    let content_inline = style
        .height
        .resolve(constraints.percent_height())
        .map_or(inline_avail, |h| style.box_sizing.content_extent(h, bp_y))
        .max(0.0);
    let content_inline = clamp_content(
        content_inline,
        style.min_height,
        style.max_height,
        constraints.percent_height(),
        bp_y,
        style.box_sizing,
    );

    // Block extent = physical width: an explicit width resolves now; an `auto`
    // width shrink-wraps the children and is decided after they are placed.
    let block_avail = (constraints.max.width - bp_x).max(0.0);
    let explicit_block = style
        .width
        .resolve(constraints.percent_width())
        .map(|w| style.box_sizing.content_extent(w, bp_x).max(0.0));
    // This box's **definite** content block size — here the physical *width* —
    // clamped into min/max-width so it is the used size, and the basis a child's
    // percentage block size resolves against (css-sizing-3.txt:1027). See the
    // horizontal path's `percent_basis`: the block axis is rotated, the defect and
    // its fix are identical. `None` (an `auto` width that shrink-wraps its
    // children) keeps the block size indefinite and leaves percentages resolving
    // against the offer, exactly as before.
    let definite_content_block = explicit_block.map(|b| {
        clamp_content(
            b,
            style.min_width,
            style.max_width,
            constraints.percent_width(),
            bp_x,
            style.box_sizing,
        )
    });
    let percent_basis = definite_content_block.map(|b| Size::new(b, content_inline));

    // Which physical edges the block-start / block-end sides map to under this
    // mode (§6.2): `vertical-rl` block-start = right, `vertical-lr` = left.
    let block_start_border = if rl { border.right } else { border.left };
    let block_end_border = if rl { border.left } else { border.right };
    let block_start_padding = if rl { padding.right } else { padding.left };
    let block_end_padding = if rl { padding.left } else { padding.right };

    let mut placed: Vec<VerticalPlacement> = Vec::with_capacity(children.len());
    // `b` is the block-end of the last placed real child (distance from the
    // block-start content edge); `pending` is the open block-axis collapsing set;
    // `gap_acc` is the additive `gap` accrued across the current boundary.
    let mut b = 0.0_f32;
    let mut pending = CollapsingMargins::EMPTY;
    let mut gap_acc = 0.0_f32;
    for (index, child) in children.iter().enumerate() {
        let cs = child.style();
        if index > 0 {
            gap_acc += style.gap;
        }
        // Block-start (leading) and block-end (trailing) margins, from the physical
        // edges the mode maps them to. Percentage margins resolve against the
        // containing block's inline size (CSS 2.1 §8.3) — the container's height.
        let (lead_margin, trail_margin) = if rl {
            (cs.margin.right, cs.margin.left)
        } else {
            (cs.margin.left, cs.margin.right)
        };
        pending = pending.adjoin(lead_margin.resolve(content_inline));

        let separation = gap_acc + pending.solve();
        let remaining_block = (block_avail - (b + separation)).max(0.0);

        // Inline (vertical) margins take from the child's offered inline space;
        // inline-start is the top, inline-end the bottom.
        let fixed_inline =
            cs.margin.top.resolve(content_inline) + cs.margin.bottom.resolve(content_inline);
        let child_inline = (content_inline - fixed_inline).max(0.0);
        // Physical constraint: width = block offer, height = inline offer — plus the
        // containing block's content box as the percentage basis when this box has a
        // definite block size, so a child's `width: 50%` reads half of *this* box
        // and not half of the room its predecessors left (see `percent_basis`).
        let child_constraints = BoxConstraints::loose(Size::new(remaining_block, child_inline));
        let size = measurer.measure_child(
            index,
            percent_basis.map_or(child_constraints, |basis| {
                child_constraints.with_percent_basis(basis)
            }),
        );

        // Inline placement (the top offset) — §10.3.3's inline solve on the
        // vertical axis, using the top/bottom margins. An over-constrained box
        // drops its inline-*end* margin per the containing block's `direction`
        // (`rtl` moves the top edge inward by the whole leftover).
        let leftover = (content_inline - fixed_inline - size.height).max(0.0);
        let inline_offset = match (cs.margin.top.is_auto(), cs.margin.bottom.is_auto()) {
            (true, true) => leftover / 2.0,
            (true, false) => leftover,
            (false, true) => cs.margin.top.resolve(content_inline),
            (false, false) => {
                let top = cs.margin.top.resolve(content_inline);
                if style.direction.is_rtl() {
                    top + leftover
                } else {
                    top
                }
            }
        };

        let block_size = size.width;

        // A zero-block-extent child with no block-axis border or padding has
        // adjoining block-start/end margins: it collapses through, exactly as a
        // zero-height child does in the horizontal path.
        let collapses_through = block_size == 0.0
            && block_start_border == 0.0
            && block_end_border == 0.0
            && block_start_padding == 0.0
            && block_end_padding == 0.0;

        if collapses_through {
            pending = pending.adjoin(trail_margin.resolve(content_inline));
            let block_start_offset = b + gap_acc + pending.solve();
            placed.push(VerticalPlacement {
                block_start_offset,
                inline_offset,
                size,
            });
            continue;
        }

        let block_start_offset = b + separation;
        placed.push(VerticalPlacement {
            block_start_offset,
            inline_offset,
            size,
        });
        b = block_start_offset + block_size;
        pending = CollapsingMargins::EMPTY.adjoin(trail_margin.resolve(content_inline));
        gap_acc = 0.0;
    }
    let children_block = b + pending.solve();

    // The used block extent: the definite one resolved (and clamped) above, else
    // the children's shrink-wrap — which is clamped here for the first time. A
    // percentage bound resolves against the containing block, like the `width`.
    let content_block = definite_content_block.unwrap_or(children_block).max(0.0);
    let content_block = clamp_content(
        content_block,
        style.min_width,
        style.max_width,
        constraints.percent_width(),
        bp_x,
        style.box_sizing,
    );

    // Map each child's block-start offset to a physical origin. `vertical-lr`
    // grows +x from the left content edge; `vertical-rl` grows −x from the right
    // content edge, so the child's left edge sits its block-start offset (plus its
    // own block size) in from the right.
    let child_boxes = placed
        .into_iter()
        .map(|p| {
            let x = if rl {
                bp_left + content_block - p.block_start_offset - p.size.width
            } else {
                bp_left + p.block_start_offset
            };
            let y = bp_top + p.inline_offset;
            LayoutBox::new(Rect::new(Point::new(x, y), p.size))
        })
        .collect();

    let border_box = Size::new(content_block + bp_x, content_inline + bp_y);
    let size = constraints.constrain_allowing_overflow(border_box);
    LayoutOutput::new(size, child_boxes)
}

#[cfg(test)]
mod tests {
    use super::FlowLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{Dimension, LayoutStyle, MarginEdges, WritingMode};
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

    // ---- vertical writing modes (css-writing-modes-4 §6.2) -----------------

    /// A fixed-size container in the given writing mode.
    fn vertical_container(mode: WritingMode) -> LayoutStyle {
        LayoutStyle {
            writing_mode: mode,
            width: Dimension::Px(300.0),
            height: Dimension::Px(300.0),
            ..LayoutStyle::default()
        }
    }

    #[test]
    fn vertical_lr_stacks_children_left_to_right_from_the_left_edge() {
        // Block axis is horizontal, block-start = left. Children advance +x by
        // their widths; inline (vertical) offset stays at the top.
        let style = vertical_container(WritingMode::VerticalLr);
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(40.0, 100.0), Size::new(60.0, 120.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        let boxes = out.child_boxes();
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        // Second child sits directly right of the first (block advance = its width).
        assert_eq!(boxes[1].origin(), Point::new(40.0, 0.0));
    }

    #[test]
    fn vertical_rl_stacks_children_right_to_left_from_the_right_edge() {
        // Block axis is horizontal, block-start = right. The first child's RIGHT
        // edge hugs the content's right edge (300); each subsequent child sits to
        // its left.
        let style = vertical_container(WritingMode::VerticalRl);
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(40.0, 100.0), Size::new(60.0, 120.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        let boxes = out.child_boxes();
        // First child: right edge at 300 → left edge at 300 − 40 = 260.
        assert_eq!(boxes[0].origin(), Point::new(260.0, 0.0));
        // Second: block offset 40 from the right, width 60 → 300 − 40 − 60 = 200.
        assert_eq!(boxes[1].origin(), Point::new(200.0, 0.0));
    }

    #[test]
    fn vertical_rl_block_start_margin_is_the_gap_from_the_right_edge() {
        // `margin-block-start` folds to `margin-right` under vertical-rl, and here
        // it is the leading gap from the block-start (right) edge: a 20px margin
        // puts the child's right edge 20px in from the content's right edge.
        let style = vertical_container(WritingMode::VerticalRl);
        let child_style = LayoutStyle {
            margin: MarginEdges::px(0.0, 20.0, 0.0, 0.0), // margin-right = 20
            ..LayoutStyle::default()
        };
        let children = [LayoutChild::new(NodeId::from_raw(1), child_style)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(100.0, 100.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        // right edge at 300 − 20 = 280 → left edge at 280 − 100 = 180.
        assert_eq!(out.child_boxes()[0].origin(), Point::new(180.0, 0.0));
    }

    #[test]
    fn vertical_lr_inline_top_margin_offsets_along_the_vertical_inline_axis() {
        // In a vertical mode the inline axis is vertical: a `margin-top` is an
        // inline-start margin and offsets the child downward, not along the block.
        let style = vertical_container(WritingMode::VerticalLr);
        let child_style = LayoutStyle {
            margin: MarginEdges::px(30.0, 0.0, 0.0, 0.0), // margin-top = 30
            ..LayoutStyle::default()
        };
        let children = [LayoutChild::new(NodeId::from_raw(1), child_style)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(50.0, 50.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        // block-start (left) offset 0, inline (top) offset 30.
        assert_eq!(out.child_boxes()[0].origin(), Point::new(0.0, 30.0));
    }

    // ---- percentage block sizes (css-sizing-3 §5.1) -------------------------

    /// Children that are themselves blocks: measuring one runs [`FlowLayout`]
    /// under the constraints the container offered it.
    ///
    /// [`FixedChildMeasurer`] cannot be used for these tests: it answers a fixed
    /// natural size and only *clamps* it into the constraints, so a percentage in
    /// the child is never resolved and the defect is invisible. This measurer is
    /// the real pipeline in miniature — the runtime's `RecMeasurer` recurses into
    /// the child's own algorithm in exactly this way.
    struct NestedBlocks(Vec<LayoutStyle>);

    impl crate::algorithm::ChildMeasurer for NestedBlocks {
        fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size {
            let style = self.0.get(index).copied().unwrap_or_default();
            FlowLayout
                .layout(&style, &[], constraints, &mut FixedChildMeasurer::default())
                .size()
        }
    }

    /// A block with a percentage height and a `100%` width — one of the two
    /// siblings of the table below.
    fn percent_height_child(percent: f32) -> LayoutStyle {
        LayoutStyle {
            width: Dimension::Percent(100.0),
            height: Dimension::Percent(percent),
            ..LayoutStyle::default()
        }
    }

    /// `.app { width: 100%; height: 100% }` in a 480×240 viewport with two block
    /// children at the given percentage heights; yields their used heights.
    fn sibling_percent_heights(first: f32, second: f32) -> (f32, f32) {
        let app = LayoutStyle {
            width: Dimension::Percent(100.0),
            height: Dimension::Percent(100.0),
            ..LayoutStyle::default()
        };
        let out = FlowLayout.layout(
            &app,
            &[child(1), child(2)],
            BoxConstraints::tight(Size::new(480.0, 240.0)),
            &mut NestedBlocks(vec![
                percent_height_child(first),
                percent_height_child(second),
            ]),
        );
        let boxes = out.child_boxes();
        (boxes[0].size().height, boxes[1].size().height)
    }

    /// Every sibling resolves its percentage block size against the *containing
    /// block*, not against the room its predecessors left.
    ///
    /// The four rows are the measured table from the external-consumer audit. Flow
    /// used to hand each child the *remaining* block extent as its available size
    /// and let the percentage resolve against that, so only the first child was
    /// right and every one after it compounded the error down the chain: the
    /// second `50%` came back 60 (50% of the 120 that remained), the second `30%`
    /// came back 50, the `80%` came back 154 — and a second `height: 100%`
    /// computed to **zero** and disappeared with no diagnostic.
    #[test]
    fn a_percentage_block_size_resolves_against_the_containing_block() {
        assert_eq!(sibling_percent_heights(50.0, 50.0), (120.0, 120.0));
        assert_eq!(sibling_percent_heights(30.0, 30.0), (72.0, 72.0));
        assert_eq!(sibling_percent_heights(20.0, 80.0), (48.0, 192.0));
        assert_eq!(sibling_percent_heights(100.0, 100.0), (240.0, 240.0));
    }

    /// css-sizing-3.txt:1027, quoted: "Otherwise, the percentage is resolved
    /// against the containing block's size. (The containing block's size is **not
    /// re-resolved** based on the resulting size of the box; the contents might
    /// thus **overflow** or underflow the containing block)."
    ///
    /// Two children at `height: 100%` of a 240px containing block are 240 each, so
    /// they stack to 480 and the second one starts at the container's bottom edge.
    /// The container stays 240: the overflow is the specified outcome, and nothing
    /// re-resolves the basis from what the first child used (which is what made the
    /// second child zero) nor clamps the children back into the parent.
    #[test]
    fn the_containing_block_is_not_re_resolved_from_a_childs_used_size() {
        let app = LayoutStyle {
            width: Dimension::Percent(100.0),
            height: Dimension::Percent(100.0),
            ..LayoutStyle::default()
        };
        let out = FlowLayout.layout(
            &app,
            &[child(1), child(2)],
            BoxConstraints::tight(Size::new(480.0, 240.0)),
            &mut NestedBlocks(vec![
                percent_height_child(100.0),
                percent_height_child(100.0),
            ]),
        );
        let boxes = out.child_boxes();
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[0].size().height, 240.0);
        // The second child overflows: it begins where the container ends.
        assert_eq!(boxes[1].origin(), Point::new(0.0, 240.0));
        assert_eq!(boxes[1].size().height, 240.0);
        // The containing block itself is untouched by that overflow.
        assert_eq!(out.size(), Size::new(480.0, 240.0));
    }

    /// A measurer that records the block-axis *offer* and the block-axis
    /// *percentage basis* each child received, and reports a fixed 100px height.
    #[derive(Default)]
    struct RecordBlockAxis(Vec<(f32, f32)>);

    impl crate::algorithm::ChildMeasurer for RecordBlockAxis {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            self.0
                .push((constraints.max.height, constraints.percent_height()));
            Size::new(constraints.max.width, 100.0)
        }
    }

    /// The two quantities that were conflated are now separate, and both still do
    /// their own job: the *offer* shrinks down the sibling chain (it is the room
    /// left for the next sibling, and what an `auto`/stretch size fills), while the
    /// *percentage basis* stays the containing block's content height throughout.
    ///
    /// The offer is unchanged in every respect, including that it starts from the
    /// space this box was *offered* (`constraints.max.height`) rather than from its
    /// own used height — which is why this test states them equal, at 240.
    #[test]
    fn the_remaining_extent_still_shrinks_while_the_percentage_basis_does_not() {
        let app = LayoutStyle::default().with_height(Dimension::Px(240.0));
        let mut measurer = RecordBlockAxis::default();
        FlowLayout.layout(
            &app,
            &[child(1), child(2)],
            BoxConstraints::tight(Size::new(480.0, 240.0)),
            &mut measurer,
        );
        assert_eq!(
            measurer.0,
            vec![(240.0, 240.0), (140.0, 240.0)],
            "offer 240 → 140 as the first child takes 100; basis 240 both times"
        );
    }

    /// An **indefinite** containing block stays indefinite, and a percentage keeps
    /// behaving exactly as it did — the separate known gap recorded in
    /// `docs/api/12-reality.md` ("percentages should resolve only against a
    /// definite size"), deliberately NOT closed here.
    ///
    /// The container's `height: auto` means there is no basis to state, so the
    /// child still resolves against the space available and the old compounding is
    /// still observable (120 then 60). Per CSS both would be `auto` — a change to
    /// the constraint model, not to this algorithm. This test exists so that fixing
    /// the definite case did not quietly move the indefinite one.
    #[test]
    fn an_indefinite_containing_block_leaves_a_percentage_as_it_was() {
        let app = LayoutStyle::default(); // height: auto
        let out = FlowLayout.layout(
            &app,
            &[child(1), child(2)],
            BoxConstraints::loose(Size::new(480.0, 240.0)),
            &mut NestedBlocks(vec![percent_height_child(50.0), percent_height_child(50.0)]),
        );
        let boxes = out.child_boxes();
        assert_eq!(
            (boxes[0].size().height, boxes[1].size().height),
            (120.0, 60.0)
        );
    }

    /// A percentage `min-height` resolves against the containing block too: the
    /// bound used to be resolved against the space the child was offered, so it
    /// compounded down the sibling chain exactly like the `height` did.
    #[test]
    fn a_percentage_min_height_bound_resolves_against_the_containing_block() {
        let app = LayoutStyle {
            width: Dimension::Percent(100.0),
            height: Dimension::Percent(100.0),
            ..LayoutStyle::default()
        };
        let floored = LayoutStyle {
            min_height: Dimension::Percent(50.0),
            ..LayoutStyle::default()
        };
        let out = FlowLayout.layout(
            &app,
            &[child(1), child(2)],
            BoxConstraints::tight(Size::new(480.0, 240.0)),
            &mut NestedBlocks(vec![floored, floored]),
        );
        let boxes = out.child_boxes();
        // 50% of 240 for BOTH, not 50% of what the first one left over.
        assert_eq!(
            (boxes[0].size().height, boxes[1].size().height),
            (120.0, 120.0)
        );
    }

    /// The mirrored path: in a vertical writing mode the block axis is physical
    /// **width**, so it is `width: 50%` that used to compound across siblings.
    ///
    /// A `vertical-lr` container 480 wide with two `width: 50%` children gives 240
    /// each — the second child starting where the first ends and touching the
    /// container's block-end edge. Before the fix the second read 50% of the 240
    /// that remained (120).
    #[test]
    fn vertical_percentage_block_size_resolves_against_the_containing_block() {
        let app = LayoutStyle {
            writing_mode: WritingMode::VerticalLr,
            width: Dimension::Px(480.0),
            height: Dimension::Px(240.0),
            ..LayoutStyle::default()
        };
        let half = LayoutStyle {
            width: Dimension::Percent(50.0),
            height: Dimension::Percent(100.0),
            ..LayoutStyle::default()
        };
        let out = FlowLayout.layout(
            &app,
            &[child(1), child(2)],
            BoxConstraints::tight(Size::new(480.0, 240.0)),
            &mut NestedBlocks(vec![half, half]),
        );
        let boxes = out.child_boxes();
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[0].size(), Size::new(240.0, 240.0));
        assert_eq!(boxes[1].origin(), Point::new(240.0, 0.0));
        assert_eq!(boxes[1].size(), Size::new(240.0, 240.0));
    }
}
