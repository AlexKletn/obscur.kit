#![doc = include_str!("flow_layout.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{Dimension, LayoutBox, LayoutStyle, WritingMode};

use super::{
    ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput, clamp_content,
    clamp_content_measured, intrinsic_inline_size,
};

#[doc = include_str!("flow_layout.doc/FlowLayout.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlowLayout;

#[doc = include_str!("flow_layout.doc/CollapsingMargins.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
struct CollapsingMargins {
    #[doc = include_str!("flow_layout.doc/max_positive.md")]
    max_positive: f32,
    #[doc = include_str!("flow_layout.doc/min_negative.md")]
    min_negative: f32,
}

impl CollapsingMargins {
    #[doc = include_str!("flow_layout.doc/EMPTY.md")]
    const EMPTY: Self = Self {
        max_positive: 0.0,
        min_negative: 0.0,
    };

    #[doc = include_str!("flow_layout.doc/adjoin.md")]
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

    #[doc = include_str!("flow_layout.doc/solve.md")]
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
        // An `auto` `height` (or a percentage against an indefinite basis) is not
        // the end of the question, because a box can be *given* a definite block
        // size without authoring one. `or_else` is that case: a TIGHT block axis is
        // an assignment, not an offer — the arrange pass hands every node the box
        // its parent's algorithm already resolved for it, and a grid or flex item
        // stretched to its area arrives here that way. css-sizing-3 §3 counts a
        // stretch-fit size as definite, so the children's percentages resolve
        // against it.
        //
        // Leaving that [`None`] is what made the toolkit's own component host
        // swallow a size: a grid cell stretched the host to 56 tall, the host's own
        // `height` stayed `auto`, and the component's `height: 100%` root behaved as
        // auto and came back at one line of text (`docs/layer-consumer-audit.md`,
        // 2-I). A LOOSE offer stays indefinite, which is the other half of the rule
        // and the reason the two are told apart by `min == max` rather than by
        // whether a number is finite.
        //
        // [`None`] survives for a genuinely indefinite axis, and this deliberately
        // does not try to invent one there — a percentage against it should compute
        // to `auto`, which is what the [`f32::INFINITY`] basis below spells.
        let definite_content_height = style
            .height
            .resolve(constraints.percent_height())
            .map(|h| {
                let content = style.box_sizing.content_extent(h, bp_y).max(0.0);
                clamp_content(
                    content,
                    style.min_height,
                    style.max_height,
                    constraints.percent_height(),
                    bp_y,
                    style.box_sizing,
                )
            })
            .or_else(|| {
                (constraints.min.height >= constraints.max.height
                    && constraints.max.height.is_finite())
                .then(|| {
                    // Clamped like the authored branch above, and for the same
                    // reason: what a child's percentage resolves against is this
                    // box's USED content height, not the raw number it was handed.
                    // Nothing upstream has already done it — a grid stretches an
                    // item to its area (`grid_layout.rs`, both-axes-`Stretch`) and
                    // `Inset::resolve_absolute_box` stretches a box pinned by
                    // `top` + `bottom`, and neither consults the box's own
                    // `min-height`/`max-height` before assigning. Re-clamping a
                    // height that was already clamped (the ordinary arrange case,
                    // where the tight box is the one this very code measured) is
                    // inert, because a min/max clamp is idempotent.
                    //
                    // Left unclamped, a box could state a containing block it does
                    // not have: assigned a zero block axis with `min-height: 40px`,
                    // it reports 40 tall — a zero tight axis is a degenerate offer
                    // that `constrain_allowing_overflow` declines to enforce — and
                    // its `height: 100%` child would have resolved against the 0.
                    clamp_content(
                        (constraints.max.height - bp_y).max(0.0),
                        style.min_height,
                        style.max_height,
                        constraints.percent_height(),
                        bp_y,
                        style.box_sizing,
                    )
                })
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
        // The basis is attached ALWAYS, and an indefinite block axis is spelled
        // [`f32::INFINITY`] rather than left unset.
        //
        // Leaving it unset is not neutral: [`BoxConstraints::percent_height`] then
        // falls back to `max.height`, which is the space *offered* to the child —
        // so `height: 50%` under an `auto`-height parent resolved against however
        // much room the child's predecessors happened to leave. css-sizing-3
        // §3.2.1 (`specs/w3c/css-sizing-3.txt:554`), vendored here, says what
        // should happen instead:
        //
        // > To have a common term for both when width/height computes to auto and
        // > when it is defined to behave as if auto were specified (as in the case
        // > of **block percentage heights resolving against an indefinite size**,
        // > see CSS2§10.5), the property is said to **behave as auto**.
        //
        // An infinite basis is exactly how that is spelled here, because
        // [`Dimension::resolve`] already answers [`None`] for a percentage against
        // a non-finite base — "no used value, fall through to the `auto` path" —
        // and every caller already handles that `None` as `auto`. So the rule
        // arrives through the existing door rather than through a new flag.
        //
        // The inline component is the containing block's content width either way
        // — the offer differs from it only by the child's own inline margins, which
        // a percentage does not subtract (CSS 2.1 §10.2).
        let percent_basis = Size::new(
            content_width,
            definite_content_height.unwrap_or(f32::INFINITY),
        );

        let mut child_boxes = Vec::with_capacity(children.len());
        // The bottom edge is in content-box coordinates and the margins adjoining
        // the current boundary collapse (CSS 2.1 §8.3.1), while the `row-gap`
        // accrued across it is additive per Box Alignment and never collapses.
        let mut last_real_child_bottom = 0.0_f32;
        let mut open_margins = CollapsingMargins::EMPTY;
        let mut accrued_gap = 0.0_f32;
        for (index, child) in children.iter().enumerate() {
            let child_style = child.style();
            if index > 0 {
                accrued_gap += style.gap;
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
            open_margins = open_margins.adjoin(child_style.margin.top.resolve(content_width));

            // For the first child the separation is just its (contained) top
            // margin, since `accrued_gap` is 0.
            let separation = accrued_gap + open_margins.solve();
            let remaining_height =
                (avail_content_height - (last_real_child_bottom + separation)).max(0.0);
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
            let child_constraints = BoxConstraints::loose(Size::new(child_width, remaining_height))
                .with_percent_basis(percent_basis);
            let child_size = measurer.measure_child(index, child_constraints);
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
            // the bottom edge nor the set is closed — so a run of empties yields
            // one margin.
            let collapses_through = child_size.height == 0.0
                && child_style.border.top == 0.0
                && child_style.border.bottom == 0.0
                && child_style.padding.top == 0.0
                && child_style.padding.bottom == 0.0;

            if collapses_through {
                open_margins =
                    open_margins.adjoin(child_style.margin.bottom.resolve(content_width));
                // Place the zero-height box at the (current) collapsed boundary;
                // it does not advance the bottom edge and does not close the set.
                let top = bp_top + last_real_child_bottom + accrued_gap + open_margins.solve();
                child_boxes.push(LayoutBox::new(Rect::new(Point::new(left, top), child_size)));
                continue;
            }

            // A real child closes the collapsing set: it sits below the
            // collapsed separation, then a fresh set opens with its bottom
            // margin and the gap accumulator resets.
            let top = last_real_child_bottom + separation;
            child_boxes.push(LayoutBox::new(Rect::new(
                Point::new(left, bp_top + top),
                child_size,
            )));
            last_real_child_bottom = top + child_size.height;
            open_margins =
                CollapsingMargins::EMPTY.adjoin(child_style.margin.bottom.resolve(content_width));
            accrued_gap = 0.0;
        }

        // Auto content height: the last real child's collapsed bottom edge plus
        // the trailing collapsing set (the last child's bottom margin — and any
        // trailing collapse-through empties — are *contained* within this box,
        // not protruded; see the module docs). With zero child margins this is
        // exactly the bottom edge, byte-identical to a gap-only stack.
        let children_content_height = last_real_child_bottom + open_margins.solve();

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

#[doc = include_str!("flow_layout.doc/VerticalPlacement.md")]
struct VerticalPlacement {
    #[doc = include_str!("flow_layout.doc/block_start_offset.md")]
    block_start_offset: f32,
    #[doc = include_str!("flow_layout.doc/inline_offset.md")]
    inline_offset: f32,
    #[doc = include_str!("flow_layout.doc/size.md")]
    size: Size,
}

#[doc = include_str!("flow_layout.doc/layout_vertical.md")]
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
    // its fix are identical — including the indefinite case. An `auto` width that
    // shrink-wraps its children leaves the block size indefinite, and an
    // indefinite basis is stated as [`f32::INFINITY`] so a percentage *behaves as
    // auto* (`specs/w3c/css-sizing-3.txt:554`) instead of resolving against the
    // offer. The physical axes are swapped here and nothing else is.
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
    let percent_basis = Size::new(
        definite_content_block.unwrap_or(f32::INFINITY),
        content_inline,
    );

    // Which physical edges the block-start / block-end sides map to under this
    // mode (§6.2): `vertical-rl` block-start = right, `vertical-lr` = left.
    let block_start_border = if rl { border.right } else { border.left };
    let block_end_border = if rl { border.left } else { border.right };
    let block_start_padding = if rl { padding.right } else { padding.left };
    let block_end_padding = if rl { padding.left } else { padding.right };

    let mut placed: Vec<VerticalPlacement> = Vec::with_capacity(children.len());
    // The block-end is a distance from the block-start content edge; the open
    // set is the block-axis analogue of the horizontal path's, and the `gap`
    // accrued across the current boundary is additive and never collapses.
    let mut last_real_child_block_end = 0.0_f32;
    let mut open_margins = CollapsingMargins::EMPTY;
    let mut accrued_gap = 0.0_f32;
    for (index, child) in children.iter().enumerate() {
        let cs = child.style();
        if index > 0 {
            accrued_gap += style.gap;
        }
        // Block-start (leading) and block-end (trailing) margins, from the physical
        // edges the mode maps them to. Percentage margins resolve against the
        // containing block's inline size (CSS 2.1 §8.3) — the container's height.
        let (lead_margin, trail_margin) = if rl {
            (cs.margin.right, cs.margin.left)
        } else {
            (cs.margin.left, cs.margin.right)
        };
        open_margins = open_margins.adjoin(lead_margin.resolve(content_inline));

        let separation = accrued_gap + open_margins.solve();
        let remaining_block = (block_avail - (last_real_child_block_end + separation)).max(0.0);

        // Inline (vertical) margins take from the child's offered inline space;
        // inline-start is the top, inline-end the bottom.
        let fixed_inline =
            cs.margin.top.resolve(content_inline) + cs.margin.bottom.resolve(content_inline);
        let child_inline = (content_inline - fixed_inline).max(0.0);
        // Physical constraint: width = block offer, height = inline offer — plus the
        // containing block's content box as the percentage basis when this box has a
        // definite block size, so a child's `width: 50%` reads half of *this* box
        // and not half of the room its predecessors left (see `percent_basis`).
        let child_constraints = BoxConstraints::loose(Size::new(remaining_block, child_inline))
            .with_percent_basis(percent_basis);
        let size = measurer.measure_child(index, child_constraints);

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
            open_margins = open_margins.adjoin(trail_margin.resolve(content_inline));
            let block_start_offset = last_real_child_block_end + accrued_gap + open_margins.solve();
            placed.push(VerticalPlacement {
                block_start_offset,
                inline_offset,
                size,
            });
            continue;
        }

        let block_start_offset = last_real_child_block_end + separation;
        placed.push(VerticalPlacement {
            block_start_offset,
            inline_offset,
            size,
        });
        last_real_child_block_end = block_start_offset + block_size;
        open_margins = CollapsingMargins::EMPTY.adjoin(trail_margin.resolve(content_inline));
        accrued_gap = 0.0;
    }
    let children_block = last_real_child_block_end + open_margins.solve();

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
    use crate::algorithm::{ChildMeasurer, FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
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

    /// A percentage block size against an **indefinite** containing block *behaves
    /// as auto* — not as a percentage of whatever was offered.
    ///
    /// `specs/w3c/css-sizing-3.txt:554` (§3.2.1 "Behaving as auto"), vendored here:
    /// "when it is defined to behave as if auto were specified (as in the case of
    /// **block percentage heights resolving against an indefinite size**, see
    /// CSS2§10.5), the property is said to behave as auto".
    ///
    /// This replaces a test that pinned the *unfixed* gap in place — it asserted
    /// the compounding `(120, 60)` that an offer-resolved percentage produced, and
    /// existed so that closing the definite case did not quietly move the
    /// indefinite one. Closing the indefinite case is what moves it, on purpose.
    ///
    /// The assertion is an **equality against the `auto` spelling**, not a
    /// hard-coded number, because "behaves as auto" is a statement about two
    /// spellings agreeing. Both children carry a 30px `min-height` floor so the
    /// agreement is not the vacuous `0 == 0`: an offer-resolved percentage would
    /// read 120 (50% of the 240 offered), and `auto` reads the content height of an
    /// empty block floored at 30.
    #[test]
    fn a_percentage_against_an_indefinite_block_size_behaves_as_auto() {
        let app = LayoutStyle::default(); // height: auto — an indefinite block size
        let floored = |height: Dimension| LayoutStyle {
            width: Dimension::Percent(100.0),
            height,
            min_height: Dimension::Px(30.0),
            ..LayoutStyle::default()
        };
        let heights = |style: LayoutStyle| {
            let out = FlowLayout.layout(
                &app,
                &[child(1), child(2)],
                BoxConstraints::loose(Size::new(480.0, 240.0)),
                &mut NestedBlocks(vec![style, style]),
            );
            let boxes = out.child_boxes();
            (boxes[0].size().height, boxes[1].size().height)
        };

        let as_percent = heights(floored(Dimension::Percent(50.0)));
        let as_auto = heights(floored(Dimension::Auto));
        assert_eq!(
            as_percent, as_auto,
            "a percentage against an indefinite basis must be the `auto` answer"
        );
        // And the number is the auto one, not the offer-resolved 120 this used to
        // produce — so the equality above is not two wrongs agreeing.
        assert_eq!(as_percent, (30.0, 30.0));
    }

    /// The same rule with no floor: an empty block whose percentage height behaves
    /// as `auto` has its content height, which is zero — and its sibling is no
    /// longer a percentage of what it left over. The old behaviour was `(120, 60)`,
    /// the compounding that made the offer-as-basis reading visible in the first
    /// place.
    #[test]
    fn an_indefinite_basis_collapses_an_empty_percentage_block_to_its_content() {
        let app = LayoutStyle::default(); // height: auto
        let out = FlowLayout.layout(
            &app,
            &[child(1), child(2)],
            BoxConstraints::loose(Size::new(480.0, 240.0)),
            &mut NestedBlocks(vec![percent_height_child(50.0), percent_height_child(50.0)]),
        );
        let boxes = out.child_boxes();
        assert_eq!((boxes[0].size().height, boxes[1].size().height), (0.0, 0.0));
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

    /// A child that is itself a flow box, so the containing block its parent
    /// states reaches something that resolves a percentage against it.
    struct Nested(LayoutStyle);

    impl ChildMeasurer for Nested {
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

    #[test]
    fn a_block_handed_its_height_is_a_definite_containing_block() {
        // The arrange pass gives every node a TIGHT box — the size its parent's
        // algorithm already resolved for it. For a box whose own `height` is
        // `auto` that assigned size is still definite: `specs/w3c/css-sizing-3.txt`
        // §3 counts a stretch-fit size as definite, so a `height: 100%` child
        // resolves against it rather than behaving as auto.
        //
        // It did not, and a component's root was the visible case: the toolkit puts
        // a host node between a grid cell and the component, the cell stretched
        // that host to 56 tall, the host's own `height` stayed `auto`, and so the
        // component's `height: 100%` root came back at the height of one line of
        // text (`docs/layer-consumer-audit.md`, 2-I).
        let full = LayoutStyle {
            height: Dimension::Percent(100.0),
            ..LayoutStyle::default()
        };
        let out = FlowLayout.layout(
            &LayoutStyle::default(),
            &[LayoutChild::new(NodeId::from_raw(1), full)],
            BoxConstraints::tight(Size::new(56.0, 56.0)),
            &mut Nested(full),
        );
        assert_eq!(out.child_boxes()[0].size().height, 56.0);
    }

    #[test]
    fn an_offered_height_is_still_indefinite() {
        // The counterpart, and the line between them: a LOOSE offer is not an
        // assignment, so `height: 100%` under it still behaves as auto and the
        // child stays at its content height. Only a tight axis is a decision the
        // parent already made.
        let full = LayoutStyle {
            height: Dimension::Percent(100.0),
            ..LayoutStyle::default()
        };
        let out = FlowLayout.layout(
            &LayoutStyle::default(),
            &[LayoutChild::new(NodeId::from_raw(1), full)],
            BoxConstraints::loose(Size::new(56.0, 56.0)),
            &mut Nested(full),
        );
        assert_eq!(out.child_boxes()[0].size().height, 0.0);
    }

    #[test]
    fn an_assigned_height_is_clamped_before_it_is_stated_as_the_containing_block() {
        // What a percentage resolves against is the box's USED content height, and
        // an assignment is not exempt from its own `min-height`/`max-height` — the
        // authored branch clamps, and nothing upstream clamps for the assigned one
        // (a grid stretches an item to its area, and a box pinned by `top` +
        // `bottom` is stretched by `Inset::resolve_absolute_box`; neither reads the
        // item's own bounds).
        //
        // A zero tight axis is the case where saying so is unarguable, because the
        // box's own report disagrees with what it told its children: a zero-height
        // assignment is a degenerate offer that `constrain_allowing_overflow`
        // declines to enforce, so `min-height: 40px` makes this box 40 tall — while
        // the unclamped assignment stated a containing block of 0, and the
        // `height: 100%` child inside a 40-tall box came back at nothing.
        let full = LayoutStyle {
            height: Dimension::Percent(100.0),
            ..LayoutStyle::default()
        };
        let host = LayoutStyle {
            min_height: Dimension::Px(40.0),
            ..LayoutStyle::default()
        };
        let out = FlowLayout.layout(
            &host,
            &[LayoutChild::new(NodeId::from_raw(1), full)],
            BoxConstraints::tight(Size::new(56.0, 0.0)),
            &mut Nested(full),
        );
        assert_eq!(out.size().height, 40.0);
        assert_eq!(out.child_boxes()[0].size().height, 40.0);
    }

    #[test]
    fn an_assigned_height_over_max_height_states_the_capped_containing_block() {
        // The other bound, same rule: `max-height` caps the used content height, so
        // that — not the 56 the parent assigned — is what the child's `height: 100%`
        // resolves against.
        let full = LayoutStyle {
            height: Dimension::Percent(100.0),
            ..LayoutStyle::default()
        };
        let host = LayoutStyle {
            max_height: Dimension::Px(30.0),
            ..LayoutStyle::default()
        };
        let out = FlowLayout.layout(
            &host,
            &[LayoutChild::new(NodeId::from_raw(1), full)],
            BoxConstraints::tight(Size::new(56.0, 56.0)),
            &mut Nested(full),
        );
        assert_eq!(out.child_boxes()[0].size().height, 30.0);
    }
}
