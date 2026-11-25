//! `FlexLayout`: single-line flex layout for the `Flex` formatting context.

use obscur_geometry::{BoxConstraints, Size};

use crate::model::{Dimension, JustifyContent, LayoutBox, LayoutStyle, Margin};

use super::content_distribution::content_distribution;
use super::flex_axis_bounds::{axis_bp, clamp_floor, main_bounds};
use super::flex_line_placement::{LineArgs, place_line};
use super::flex_main_margins::reserved;
use super::flex_main_probe::{content_max_main, intrinsic_main};
use super::layout_child::ordered_child_indices;
use super::size_containment::contained_style;
use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput, clamp_content};

#[doc = include_str!("../../docs/api/algorithm/flex_layout.doc/FlexLayout.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlexLayout;

impl LayoutAlgorithm for FlexLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let contained = contained_style(style.clone(), constraints);
        let style = contained.as_ref().unwrap_or(style);
        let is_row = style.flex_direction.is_row();
        let is_reverse = style.flex_direction.is_reverse();

        let border = style.border;
        let padding = style.padding;
        let bp_left = border.left + padding.left;
        let bp_top = border.top + padding.top;
        let bp_x = bp_left + border.right + padding.right;
        let bp_y = bp_top + border.bottom + padding.bottom;

        // Step 1: container content extents, split into main / cross. An explicit
        // `width` / `height` on the container is honoured (resolved against the
        // available max, so a fixed px or a percentage of it both work); an `Auto`
        // dimension falls back to the available content extent, so the common
        // fill-available case is byte-identical to before.
        //
        // `box-sizing` governs how an explicit size maps to the content extent:
        // `content-box` (the default) takes it as-is, `border-box` subtracts the
        // axis border + padding (floored at `0`). The `Auto` branch already
        // subtracts border + padding, so both modes are consistent — no phantom
        // inflation and no clamp mismatch when an explicitly-sized container is
        // arranged at its authored (border-box) size by its parent.
        // A percentage resolves against the **containing block**, which the
        // caller states; `max` is the space available for an `auto` size and is
        // the fallback when no basis was given. Conflating the two let an
        // arranged box read its own border box as its containing block.
        let content_width = style
            .width
            .resolve(constraints.percent_width())
            .map_or(constraints.max.width - bp_x, |w| {
                style.box_sizing.content_extent(w, bp_x)
            })
            .max(0.0);
        let content_height = style
            .height
            .resolve(constraints.percent_height())
            .map_or(constraints.max.height - bp_y, |h| {
                style.box_sizing.content_extent(h, bp_y)
            })
            .max(0.0);
        let resolve_gap = |axis: &Option<Dimension>, basis| {
            axis.as_ref()
                .or(style.gap_dimension.as_ref())
                .map_or(style.gap, |gap| gap.resolve(basis).unwrap_or(0.0).max(0.0))
        };
        let horizontal_gap = resolve_gap(
            &style.column_gap,
            if content_width.is_finite() {
                content_width
            } else {
                0.0
            },
        );
        let height_definite = style.height.resolve(constraints.percent_height()).is_some()
            || (constraints.max.height.is_finite()
                && constraints.min.height >= constraints.max.height);
        let vertical_gap = resolve_gap(
            &style.row_gap,
            if height_definite { content_height } else { 0.0 },
        );
        let (gap, cross_gap) = if is_row {
            (horizontal_gap, vertical_gap)
        } else {
            (vertical_gap, horizontal_gap)
        };
        // An **unbounded** main axis is the max-content *question*, and the honest
        // answer is what the items need — not the infinity they were offered.
        // `content_width` is `constraints.max.width - bp` for a `width: auto`
        // container, which is the *block* answer: "as much as I was given". A flex
        // container answering that under an infinite offer reports an infinite
        // max-content, and every caller asking how wide it wants to be gets a
        // number it cannot use — which is how a nested flex item came to swallow a
        // whole flex line.
        //
        // The answer must be summed from the items, never read off their placed
        // positions: `justify-content` has already spread them across the infinite
        // line, so the rightmost edge is infinity too.
        let main_unbounded = if is_row {
            constraints.max.width.is_infinite()
                && !style.size_containment.contains_width(style.writing_mode)
        } else {
            constraints.max.height.is_infinite()
                && !style.size_containment.contains_height(style.writing_mode)
        };
        let (container_main, container_cross) = if is_row {
            (content_width, content_height)
        } else {
            (content_height, content_width)
        };

        // Whether the block (height) extent is content-sized rather than
        // fill-available. In horizontal writing mode `height: auto` hugs its
        // content, but only when the incoming height constraint is *indefinite*
        // (a loose measure, `min.height != max.height`, or an unbounded axis). A
        // *definite* (tight) height from the parent — the constraint every
        // `arrange` pass supplies — fills as before, so placement is unchanged and
        // only the measured size a parent observes shrinks to fit.
        //
        // For a column this shrinks the MAIN axis to the stacked children (the
        // previous "fill the whole available height" was the root of nested
        // flex-column items over-reporting their height); for a row it shrinks the
        // CROSS axis to the tallest item. Width (the inline axis) always fills.
        let height_is_auto = style.height.resolve(constraints.max.height).is_none();
        let height_indefinite = height_is_auto
            && (!constraints.max.height.is_finite()
                || (constraints.max.height - constraints.min.height).abs() > f32::EPSILON);

        // The flex base size uses border-box extents before asking the child
        // measurer. Resolve these edges with the same basis as that measurer.
        let resolved_children = if children
            .iter()
            .any(|child| child.style.padding_lengths.is_some())
        {
            let mut resolved = children.to_vec();
            for child in &mut resolved {
                let basis = if child.style.writing_mode.is_vertical() {
                    content_height
                } else {
                    content_width
                };
                child.style.resolve_padding(basis);
            }
            Some(resolved)
        } else {
            None
        };
        let children = resolved_children.as_deref().unwrap_or(children);
        let count = children.len();

        // Per-item margins split into main-leading / main-trailing (which consume
        // main-axis space and offset items) and a cross-leading offset. The
        // physical edge that leads in cursor order depends on direction and
        // reversal; a zero-margin item contributes nothing on every edge, so this
        // is a no-op for the common case.
        let mut main_lead = Vec::with_capacity(count);
        let mut main_trail = Vec::with_capacity(count);
        let mut cross_lead = Vec::with_capacity(count);
        for child in children {
            // Edges stay [`Margin`] rather than collapsing to lengths here, because
            // an `auto` edge has two answers downstream: it *reserves* nothing
            // (§8.1) but it *absorbs* free space (see `place_line`). Flattening it
            // to `0` now would throw the second one away.
            //
            // Percentages, on the other hand, resolve right here: CSS 2.1 §8.3
            // resolves every margin edge against the containing block's INLINE size
            // — this container's content width — regardless of which way the flex
            // axis runs. It is known, it cannot change, and resolving it once leaves
            // only `Px` and `Auto` for the rest of the algorithm to think about.
            let fix = |edge: &Margin| match edge {
                Margin::Percent(_) | Margin::Math(_) => Margin::Px(edge.resolve(content_width)),
                px_or_auto => px_or_auto.clone(),
            };
            let m = &child.style.margin;
            let (lead, trail) = if is_row {
                if is_reverse {
                    (&m.right, &m.left)
                } else {
                    (&m.left, &m.right)
                }
            } else if is_reverse {
                (&m.bottom, &m.top)
            } else {
                (&m.top, &m.bottom)
            };
            main_lead.push(fix(lead));
            main_trail.push(fix(trail));
            cross_lead.push(fix(if is_row { &m.top } else { &m.left }));
        }

        // Step 2: per item, the **flex base size** (`flex_basis`/`width`/measured,
        // never negative) and the **hypothetical main size** (the flex base clamped
        // into the item's main-axis min/max range). The freeze loop (§9.7) needs
        // both: the unclamped base is what grow/shrink distribute from, while the
        // clamped hypothetical drives line-breaking and the freeze-inflexible test.
        let mut flex_base = Vec::with_capacity(count);
        let mut base_main = Vec::with_capacity(count);
        // Per-item main-axis min floor / max ceiling, resolved to pixels. `main_max`
        // is `f32::INFINITY` when the item has no `max`. `main_min` carries the
        // §4.5 content-based automatic minimum when the item's `min` is `auto` /
        // unset, and the explicit `min` otherwise. Both the `base_main` clamp below
        // and the §9.7 freeze loop consume these so the automatic minimum acts as
        // the true shrink floor.
        let mut main_min = Vec::with_capacity(count);
        let mut main_max = Vec::with_capacity(count);
        // Items whose §4.5 automatic minimum is a `content.min(fb)` floor (row main
        // axis, or any definite basis): that floor is `<= fb`, so it never raises the
        // item's base size and can only bite when the item is actually shrunk below
        // it. Its `content_max_main` measure is therefore DEFERRED to the per-line
        // shrink gate in `place_line` (resolved only for lines under real shrink
        // pressure). `false` means the item's floor was already resolved eagerly.
        let mut deferred_floor = vec![false; count];
        for (index, child) in children.iter().enumerate() {
            let main_dim = if is_row {
                &child.style.width
            } else {
                &child.style.height
            };
            // An explicit main size (from `flex-basis` or `width`/`height`) is
            // authored against the item's box-sizing, so it is expanded to the
            // border-box extent the item occupies (`content-box` adds the item's
            // main-axis border + padding). The measured fallback already reports a
            // border box, so it is used as-is.
            //
            // Each source is asked for an INTRINSIC answer before an arithmetic
            // one, because `Dimension::resolve` cannot tell the two apart: it
            // answers `None` for `min-content` exactly as it does for `auto`, so
            // an unasked keyword falls through to the next source and, in the end,
            // to the natural-size fallback — `flex-basis: max-content` meaning
            // `flex-basis: auto`. `intrinsic_main` measures nothing for a
            // non-keyword, so the common item costs what it always did.
            let explicit_main = |explicit: f32| {
                child
                    .style
                    .box_sizing
                    .border_extent(explicit, axis_bp(&child.style, is_row))
            };
            let basis = &child.style.flex_basis;
            let base = if let Some(px) = intrinsic_main(
                &mut *measurer,
                index,
                basis,
                is_row,
                container_main,
                container_cross,
            ) {
                px
            } else if let Some(explicit) = basis.resolve(container_main) {
                explicit_main(explicit)
            } else if let Some(px) = intrinsic_main(
                &mut *measurer,
                index,
                main_dim,
                is_row,
                container_main,
                container_cross,
            ) {
                px
            } else if let Some(explicit) = main_dim.resolve(container_main) {
                explicit_main(explicit)
            } else {
                // §9.2.3 step E: an `auto` basis with no definite main size
                // resolves to the item's **max-content** size — what it needs,
                // not what it was offered.
                //
                // Offering the container's content box instead is the same
                // question a block child answers by *stretching*, so every plain
                // item came back the width of the line. One such item then
                // consumed the whole free space and starved the siblings that
                // had declared `flex-grow`, which is the inverse of what those
                // declarations mean.
                //
                // A fill-available item has no finite max-content; the line's
                // own extent is the only honest answer for it, and grow/shrink
                // then distribute from there as before.
                let max_content = content_max_main(&mut *measurer, index, is_row, container_cross);
                if max_content.is_finite() {
                    max_content
                } else {
                    container_main
                }
            };
            let (min_main, max_main) = main_bounds(&child.style, is_row);
            let fb = base.max(0.0);
            let max_val = max_main.resolve(container_main).unwrap_or(f32::INFINITY);
            // §4.5 automatic minimum size: an `auto` (or unset) main-axis `min` does
            // not floor at 0 — its used value is the item's content-based minimum
            // size. An EXPLICIT `min` is used verbatim (byte-identical to before).
            // The extra `content_max_main` measure is computed LAZILY: it runs only
            // when it can actually change the result, so the common frame takes zero
            // extra measures.
            let min_floor = if let Some(explicit) = min_main.resolve(container_main) {
                explicit
            } else if child.style.scroll_container {
                // §4.5: "for scroll containers the automatic minimum size is zero"
                // (css-flexbox-1.txt:1084-1085). An item whose main-axis overflow is
                // scrollable (`scroll`/`auto`/`hidden`) drops the content-based floor
                // entirely, so it may shrink all the way to 0. `deferred_floor` stays
                // `false`, so `place_line` never re-introduces a content floor.
                0.0
            } else {
                let has_definite = child
                    .style
                    .flex_basis
                    .resolve(container_main)
                    .or_else(|| main_dim.resolve(container_main))
                    .is_some();
                if !is_row && !has_definite {
                    // Block (column) main axis with an auto basis: the block does not
                    // reflow, so max-content == min-content and the content extent is
                    // the exact automatic minimum. It differs from `fb` ONLY when `fb`
                    // was clamped by a finite, undersized container main (the
                    // card-collapse case) — then the natural size may exceed the
                    // clamped `fb`, so the floor must be measured. When the container
                    // main is unbounded, or `fb` sits below it, `natural == fb`: use
                    // `fb` as the floor with NO extra measure.
                    if container_main.is_finite() && fb >= container_main {
                        let content =
                            content_max_main(&mut *measurer, index, is_row, container_cross);
                        content.min(max_val).max(0.0)
                    } else {
                        fb.min(max_val).max(0.0)
                    }
                } else {
                    // Inline (row) main axis, or a definite basis: the floor is
                    // `content.min(fb) <= fb`, so it never inflates `base_main` and can
                    // only bite when the item is actually shrunk below it. Defer the
                    // content measure to `place_line`, which resolves it only for lines
                    // under real shrink pressure; 0 is the pre-§4.5 baseline used
                    // otherwise (output-identical whenever the line does not shrink,
                    // and it also covers the fill-available `!content.is_finite()` →
                    // no-floor case).
                    deferred_floor[index] = true;
                    0.0
                }
            };
            flex_base.push(fb);
            base_main.push(clamp_floor(fb, min_floor, max_val));
            main_min.push(min_floor);
            main_max.push(max_val);
        }

        // Steps 3–8: break the items into flex lines, then size / justify /
        // align each line independently and stack the lines along the cross axis.
        //
        // `flex-wrap: nowrap` (the default) has exactly one line holding every
        // item; it is byte-for-byte identical to the historical single-line path
        // (`stretch_target = Some(container_cross)` reproduces the old
        // fill-the-container stretch and container-relative alignment).
        let mut child_boxes = vec![LayoutBox::default(); count];
        let order = ordered_child_indices(children);

        if !style.flex_wrap.is_multi_line() {
            let line = order.unwrap_or_else(|| (0..count).collect());

            // The main-axis extent the single line's items occupy: their base main
            // sizes, the inter-item gaps, and their main-axis margins. This is the
            // content main size a column hugs to under an indefinite height.
            let gap_span = gap * (count.saturating_sub(1) as f32);
            let sum_base: f32 = base_main.iter().sum();
            let sum_margin_main: f32 = main_lead
                .iter()
                .zip(main_trail.iter())
                .map(|(l, t)| reserved(l) + reserved(t))
                .sum();
            let content_main = sum_base + gap_span + sum_margin_main;

            let placed = place_line(LineArgs {
                line: &line,
                base_main: &base_main,
                flex_base: &flex_base,
                main_min: &main_min,
                main_max: &main_max,
                deferred_floor: &deferred_floor,
                children,
                style,
                is_row,
                is_reverse,
                line_main: if is_reverse && !is_row && height_indefinite {
                    content_main.max(0.0)
                } else {
                    container_main
                },
                gap,
                main_lead: &main_lead,
                main_trail: &main_trail,
                cross_lead: &cross_lead,
                stretch_target: Some(container_cross),
                container_cross,
                bp_left,
                bp_top,
                cross_offset: 0.0,
                // The content cross extent is only needed to hug the (auto,
                // indefinite) height of a ROW; otherwise skip the extra
                // pre-stretch measurement so the definite path stays identical.
                content_cross_needed: height_indefinite && is_row,
                line_cross_stretch: None,
                percent_basis: Size::new(content_width, content_height),
                measurer: &mut *measurer,
                child_boxes: &mut child_boxes,
            });

            // Content-box height: `auto` + indefinite hugs its content (a column's
            // stacked main extent, a row's tallest item); otherwise fill the
            // resolved/available height exactly as before.
            let final_content_height = if height_indefinite {
                if is_row {
                    placed.content_cross
                } else {
                    content_main
                }
            } else {
                content_height
            };

            // Clamp the container's OWN box into its authored min/max (spec §4.2)
            // — e.g. `min-height: 100vh` on a flex column fills a tall viewport.
            // Same rule as the multi-line tail below: an unbounded main axis is
            // asking what the content needs, and `content_main` — the items' base
            // sizes plus gaps and margins — is exactly that answer.
            let own_main = if main_unbounded {
                content_main.max(0.0)
            } else if is_row {
                content_width
            } else {
                final_content_height
            };
            let own_w = clamp_content(
                if is_row { own_main } else { content_width },
                style.min_width.clone(),
                style.max_width.clone(),
                constraints.max.width,
                bp_x,
                style.box_sizing,
            );
            let own_h = clamp_content(
                if is_row {
                    final_content_height
                } else {
                    own_main
                },
                style.min_height.clone(),
                style.max_height.clone(),
                constraints.max.height,
                bp_y,
                style.box_sizing,
            );
            let border_box = Size::new(own_w + bp_x, own_h + bp_y);
            return LayoutOutput::new(constraints.constrain(border_box), child_boxes);
        }

        // Multi-line: greedily fill lines along the main axis. An item (with its
        // main-axis margins) that does not fit on the current non-empty line
        // starts a new one; an item wider than the container occupies its own line.
        let mut lines: Vec<Vec<usize>> = Vec::new();
        let mut current: Vec<usize> = Vec::new();
        let mut current_main = 0.0_f32;
        for position in 0..count {
            let index = order.as_ref().map_or(position, |indices| indices[position]);
            let item_main =
                base_main[index] + reserved(&main_lead[index]) + reserved(&main_trail[index]);
            let with_item = if current.is_empty() {
                item_main
            } else {
                current_main + gap + item_main
            };
            if !current.is_empty() && with_item > container_main {
                lines.push(std::mem::take(&mut current));
                current_main = item_main;
            } else {
                current_main = with_item;
            }
            current.push(index);
        }
        if !current.is_empty() {
            lines.push(current);
        }

        // `wrap-reverse` stacks the lines in the opposite cross-axis order: the
        // last line leads at the cross start.
        if style.flex_wrap.is_reverse() {
            lines.reverse();
        }

        // Whether the container's cross size is content-sized (`auto`) rather than
        // a definite extent. `align-content` only redistributes leftover cross
        // space, so it is inert on an `auto` cross (the container hugs its lines and
        // there is nothing left over).
        let cross_is_auto = if is_row {
            style.height.resolve(constraints.max.height).is_none()
        } else {
            style.width.resolve(constraints.max.width).is_none()
        };

        // Explicit start alignment packs without growing the lines. Normal
        // alignment behaves like stretch in flex, so it needs the natural
        // line sizes to distribute the leftover cross space.
        let needs_align_content =
            !cross_is_auto && style.align_content != JustifyContent::FlexStart;

        // A place-one-line closure so the packing and the align-content pass share
        // exactly one call shape (only the cross offset and the stretch target differ).
        let place = |line: &[usize],
                     cross_offset: f32,
                     line_cross_stretch: Option<f32>,
                     boxes: &mut [LayoutBox],
                     measurer: &mut dyn ChildMeasurer|
         -> f32 {
            place_line(LineArgs {
                line,
                base_main: &base_main,
                flex_base: &flex_base,
                main_min: &main_min,
                main_max: &main_max,
                deferred_floor: &deferred_floor,
                children,
                style,
                is_row,
                is_reverse,
                line_main: container_main,
                gap,
                main_lead: &main_lead,
                main_trail: &main_trail,
                cross_lead: &cross_lead,
                stretch_target: None,
                container_cross,
                bp_left,
                bp_top,
                cross_offset,
                content_cross_needed: false,
                line_cross_stretch,
                percent_basis: Size::new(content_width, content_height),
                measurer,
                child_boxes: boxes,
            })
            .cross
        };

        // §8.4 align-content: leading cross offset, extra spacing between adjacent
        // lines, and a per-line cross growth (only `stretch` grows lines). Computed
        // from the natural line crosses when a non-default distribution is asked for
        // on a definite cross; otherwise all zero (the packing default).
        let (ac_lead, ac_between, ac_grow, natural) = if needs_align_content {
            let mut natural = Vec::with_capacity(lines.len());
            for line in &lines {
                natural.push(place(line, 0.0, None, &mut child_boxes, &mut *measurer));
            }
            let n = lines.len();
            let total_natural: f32 = natural.iter().sum();
            let total_gap = cross_gap * (n.saturating_sub(1) as f32);
            let free = container_cross - total_natural - total_gap;
            let (lead, between) = content_distribution(n, free, style.align_content);
            let grow = if free.is_finite()
                && free > 0.0
                && n > 0
                && matches!(
                    style.align_content,
                    JustifyContent::Normal | JustifyContent::Stretch
                ) {
                free / n as f32
            } else {
                0.0
            };
            (lead, between, grow, natural)
        } else {
            (0.0, 0.0, 0.0, Vec::new())
        };

        // Stack each line at the running cross offset; the cross-axis gap (plus any
        // `align-content` spacing) sits between adjacent lines. The container's cross
        // extent grows to the sum of the line cross sizes when it is `auto`.
        let mut cross_offset = ac_lead;
        let mut total_cross = 0.0_f32;
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                cross_offset += cross_gap + ac_between;
                total_cross += cross_gap;
            }
            let stretch = if ac_grow > 0.0 {
                Some(natural[i] + ac_grow)
            } else {
                None
            };
            let line_cross = place(
                line,
                cross_offset,
                stretch,
                &mut child_boxes,
                &mut *measurer,
            );
            cross_offset += line_cross;
            total_cross += line_cross;
        }
        let main_content = if main_unbounded {
            // Nothing wraps on an unbounded axis, so this is one line's worth —
            // computed the same way the single-line path computes it.
            let gap_span = gap * (count.saturating_sub(1) as f32);
            let sum_base: f32 = base_main.iter().sum();
            let sum_margin_main: f32 = main_lead
                .iter()
                .zip(main_trail.iter())
                .map(|(l, t)| reserved(l) + reserved(t))
                .sum();
            (sum_base + gap_span + sum_margin_main).max(0.0)
        } else {
            container_main
        };

        let (final_content_width, final_content_height) = if is_row {
            let h = if cross_is_auto {
                total_cross
            } else {
                content_height
            };
            (main_content, h)
        } else {
            let w = if cross_is_auto {
                total_cross
            } else {
                content_width
            };
            (w, main_content)
        };

        // Clamp the container's OWN box into its authored min/max (spec §4.2).
        let own_w = clamp_content(
            final_content_width,
            style.min_width.clone(),
            style.max_width.clone(),
            constraints.max.width,
            bp_x,
            style.box_sizing,
        );
        let own_h = clamp_content(
            final_content_height,
            style.min_height.clone(),
            style.max_height.clone(),
            constraints.max.height,
            bp_y,
            style.box_sizing,
        );
        let border_box = Size::new(own_w + bp_x, own_h + bp_y);
        LayoutOutput::new(constraints.constrain(border_box), child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::FlexLayout;
    use crate::algorithm::{ChildMeasurer, FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{
        AlignItems, AlignSelf, BoxSizing, Dimension, FlexDirection, FlexWrap, JustifyContent,
        LayoutDisplay, LayoutStyle,
    };
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    /// Container style with the given direction/justify/align and gap.
    fn container(
        direction: FlexDirection,
        justify: JustifyContent,
        align: AlignItems,
        gap: f32,
    ) -> LayoutStyle {
        LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_direction: direction,
            justify_content: justify,
            align_items: align,
            gap,
            ..LayoutStyle::default()
        }
    }

    /// An item with a fixed flex-basis, no growth, no shrink.
    fn fixed_basis(raw: u64, basis: f32) -> LayoutChild {
        LayoutChild::new(
            NodeId::from_raw(raw),
            LayoutStyle {
                display: LayoutDisplay::Flex,
                flex_basis: Dimension::Px(basis),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..LayoutStyle::default()
            },
        )
    }

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected {b}, got {a}");
    }

    // ---- per-item align-self + cross min/max (audit Tier-1 §4) --------------

    /// A fixed-basis item carrying a per-item `align-self` override.
    fn with_self(raw: u64, basis: f32, align: AlignSelf) -> LayoutChild {
        let LayoutChild { node, style } = fixed_basis(raw, basis);
        LayoutChild::new(node, style.with_align_self(align))
    }

    #[test]
    fn align_self_overrides_the_containers_align_items() {
        // The container stretches, but item 2 opts out with `align-self: center`
        // and item 3 with `flex-end`. Flex read only the container's `align-items`,
        // so both overrides were silently dropped — while grid honoured them.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [
            fixed_basis(1, 50.0),
            with_self(2, 50.0, AlignSelf::Center),
            with_self(3, 50.0, AlignSelf::End),
        ];
        // The measured (hypothetical) cross size of every item is 20 tall.
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(50.0, 20.0),
            Size::new(50.0, 20.0),
            Size::new(50.0, 20.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Item 1 still inherits `stretch`: fills the 100 cross at y=0.
        assert_eq!(b[0].size().height, 100.0);
        approx(b[0].origin().y, 0.0);
        // Item 2 centers its own 20-tall box: (100-20)/2 = 40.
        assert_eq!(b[1].size().height, 20.0);
        approx(b[1].origin().y, 40.0);
        // Item 3 sits at the cross end: 100-20 = 80.
        assert_eq!(b[2].size().height, 20.0);
        approx(b[2].origin().y, 80.0);
    }

    #[test]
    fn align_self_stretch_opts_into_stretch_under_a_flex_start_container() {
        // The override works in both directions: a `flex-start` container leaves
        // items at their measured cross, but `align-self: stretch` fills.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), with_self(2, 50.0, AlignSelf::Stretch)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 20.0), Size::new(50.0, 20.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(
            b[0].size().height,
            20.0,
            "inherits flex-start: measured cross"
        );
        assert_eq!(
            b[1].size().height,
            100.0,
            "align-self: stretch fills the cross"
        );
    }

    #[test]
    fn a_stretched_item_stops_at_its_own_cross_max() {
        // `align-items: stretch` would fill the 100 cross, but the item's own
        // `max-height: 40` caps it (§9.4 step 11). The cross axis dropped its
        // min/max entirely while the main axis always applied them.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let LayoutChild { node, style: item } = fixed_basis(1, 50.0);
        let children = [LayoutChild::new(
            node,
            LayoutStyle {
                max_height: Dimension::Px(40.0),
                ..item
            },
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(50.0, 20.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].size().height, 40.0);
    }

    #[test]
    fn a_measured_item_is_floored_by_its_own_cross_min() {
        // A 20-tall measured item under a `flex-start` container is floored to its
        // own `min-height: 60`.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let LayoutChild { node, style: item } = fixed_basis(1, 50.0);
        let children = [LayoutChild::new(
            node,
            LayoutStyle {
                min_height: Dimension::Px(60.0),
                ..item
            },
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(50.0, 20.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].size().height, 60.0);
    }

    #[test]
    fn a_column_clamps_its_cross_max_width() {
        // The cross axis is width in a column: `max-width: 30` caps a stretch.
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let LayoutChild { node, style: item } = fixed_basis(1, 50.0);
        let children = [LayoutChild::new(
            node,
            LayoutStyle {
                max_width: Dimension::Px(30.0),
                ..item
            },
        )];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(20.0, 50.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].size().width, 30.0);
    }

    #[test]
    fn row_justify_flex_start_places_items_left() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let mut measurer = FixedChildMeasurer::default();
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(50.0, 0.0));
        // Stretch fills the cross axis.
        assert_eq!(b[0].size(), Size::new(50.0, 100.0));
        assert_eq!(out.size(), Size::new(200.0, 100.0));
    }

    #[test]
    fn row_justify_variants_place_items() {
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let constraints = BoxConstraints::tight(Size::new(200.0, 100.0));

        // Center: leading = remaining/2 = 50.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::Center,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, 50.0);
        approx(out.child_boxes()[1].origin().x, 100.0);

        // FlexEnd: leading = remaining = 100.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexEnd,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, 100.0);
        approx(out.child_boxes()[1].origin().x, 150.0);

        // SpaceBetween: spacing = remaining/(2-1) = 100.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::SpaceBetween,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, 0.0);
        approx(out.child_boxes()[1].origin().x, 150.0);

        // SpaceEvenly: s = 100/3; item0 at s, item1 at 50 + 2s.
        let s = 100.0_f32 / 3.0;
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::SpaceEvenly,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].origin().x, s);
        approx(out.child_boxes()[1].origin().x, 50.0 + 2.0 * s);
    }

    #[test]
    fn flex_grow_splits_positive_free_space() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 50.0);
        a.style.flex_grow = 1.0;
        let mut b = fixed_basis(2, 50.0);
        b.style.flex_grow = 3.0;
        let children = [a, b];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        // free = 100, split 1:3 → 25 and 75 added.
        let boxes = out.child_boxes();
        approx(boxes[0].size().width, 75.0);
        approx(boxes[1].size().width, 125.0);
        approx(boxes[0].origin().x, 0.0);
        approx(boxes[1].origin().x, 75.0);
    }

    #[test]
    fn flex_shrink_absorbs_overflow() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 80.0);
        a.style.flex_shrink = 1.0;
        let mut b = fixed_basis(2, 80.0);
        b.style.flex_shrink = 1.0;
        let children = [a, b];
        // container main = 100, total base = 160, overflow = 60 split evenly.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        approx(boxes[0].size().width, 50.0);
        approx(boxes[1].size().width, 50.0);
        approx(boxes[1].origin().x, 50.0);
    }

    #[test]
    fn align_stretch_vs_center_sets_cross() {
        let children = [fixed_basis(1, 50.0)];
        let constraints = BoxConstraints::tight(Size::new(200.0, 100.0));

        // Stretch → item cross fills the container, at cross start 0.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::new(vec![Size::new(50.0, 40.0)]),
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(out.child_boxes()[0].size(), Size::new(50.0, 100.0));

        // Center → measured cross 40, centred: (100 - 40)/2 = 30.
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::Center,
                0.0,
            ),
            &children,
            constraints,
            &mut FixedChildMeasurer::new(vec![Size::new(50.0, 40.0)]),
        );
        approx(out.child_boxes()[0].origin().y, 30.0);
        approx(out.child_boxes()[0].size().height, 40.0);
    }

    #[test]
    fn column_direction_swaps_axes() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 200.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        // Items stack vertically; cross (width) stretched to 100.
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[0].size(), Size::new(100.0, 50.0));
        assert_eq!(boxes[1].origin(), Point::new(0.0, 50.0));
        assert_eq!(boxes[1].size(), Size::new(100.0, 50.0));
    }

    #[test]
    fn container_min_height_floors_its_own_box() {
        // A flex column whose content is 50 tall, under loose constraints, is
        // floored to `min-height: 300` — the §4.2 clamp flex used to drop for the
        // container's own box (it applied min/max only to items).
        let mut style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        style.min_height = Dimension::Px(300.0);
        let children = [fixed_basis(1, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut FixedChildMeasurer::default(),
        );
        assert_eq!(out.size().height, 300.0);
    }

    #[test]
    fn gap_is_honored() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            10.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        approx(boxes[0].origin().x, 0.0);
        // 50 (item0) + 10 (gap) = 60.
        approx(boxes[1].origin().x, 60.0);
    }

    #[test]
    fn row_reverse_reverses_order() {
        let style = container(
            FlexDirection::RowReverse,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let boxes = out.child_boxes();
        // The first child starts at the right edge; the group leaves space on the left.
        approx(boxes[0].origin().x, 150.0);
        approx(boxes[1].origin().x, 100.0);
    }

    #[test]
    fn explicit_row_width_bounds_main_axis() {
        // A `width: 200px` row under a 500px available extent packs items against
        // its OWN 200px, not the 500px available.
        let mut style = container(
            FlexDirection::Row,
            JustifyContent::FlexEnd,
            AlignItems::FlexStart,
            0.0,
        );
        style.width = Dimension::Px(200.0);
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        // Container is its authored 200px, not the 500px available.
        approx(out.size().width, 200.0);
        let b = out.child_boxes();
        // justify flex-end: remaining = 200 - 100 = 100, so items pack to x=100/150.
        approx(b[0].origin().x, 100.0);
        approx(b[1].origin().x, 150.0);
    }

    #[test]
    fn explicit_column_height_bounds_main_axis() {
        // A `height: 200px` column under a 500px available extent packs items
        // against its OWN 200px along the (vertical) main axis.
        let mut style = container(
            FlexDirection::Column,
            JustifyContent::FlexEnd,
            AlignItems::FlexStart,
            0.0,
        );
        style.height = Dimension::Px(200.0);
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 500.0)),
            &mut FixedChildMeasurer::default(),
        );
        approx(out.size().height, 200.0);
        let b = out.child_boxes();
        // flex-end along the vertical main axis: items pack to y=100/150 within 200.
        approx(b[0].origin().y, 100.0);
        approx(b[1].origin().y, 150.0);
    }

    #[test]
    fn auto_width_fills_available_regression() {
        // With no explicit width, the row still fills the available extent (500),
        // preserving the fill-available default.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexEnd,
            AlignItems::FlexStart,
            0.0,
        );
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        approx(out.size().width, 500.0);
        let b = out.child_boxes();
        // flex-end within the full 500: remaining = 500 - 100 = 400.
        approx(b[0].origin().x, 400.0);
        approx(b[1].origin().x, 450.0);
    }

    #[test]
    fn border_box_centers_child_in_padded_toolbar() {
        // Regression (the "chips pressed to the bottom" toolbar bug): a
        // `height: 52; padding: 8; align-items: center` flex row with
        // `box-sizing: border-box` and a 28px-tall child. The parent arranges the
        // container tight at its authored (border-box) 52px, so the container's
        // own layout must derive its content region as `52 - 16 = 36` and centre
        // the child within it — NOT centre against the full 52 and then overflow
        // the bottom when the box is clamped back to 52.
        let style = LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            height: Dimension::Px(52.0),
            padding: Insets::uniform(8.0),
            box_sizing: BoxSizing::BorderBox,
            ..LayoutStyle::default()
        };
        let children = [fixed_basis(1, 96.0)];
        // The child measures 28px tall on the cross axis.
        let out = FlexLayout.layout(
            &style,
            &children,
            // The parent arranges an explicitly-sized flex item tight at its
            // authored border-box size (52px tall here).
            BoxConstraints::tight(Size::new(300.0, 52.0)),
            &mut FixedChildMeasurer::new(vec![Size::new(96.0, 28.0)]),
        );
        // The container's own border box is exactly the authored 52 (no phantom
        // inflation to 68, no clamp mismatch).
        approx(out.size().height, 52.0);
        let child = out.child_boxes()[0];
        // Content region is `padding.top .. padding.top + 36` = 8..44; the 28px
        // child is centred there: top = 8 + (36 - 28) / 2 = 12.
        approx(child.origin().y, 12.0);
        approx(child.size().height, 28.0);
        // Equal gap above and below (12 - 8 == 44 - 40 == 4), and no overflow of
        // the container's border box.
        let gap_above = child.origin().y - style.padding.top;
        let content_bottom = style.padding.top + 36.0;
        let gap_below = content_bottom - (child.origin().y + child.size().height);
        approx(gap_above, gap_below);
        assert!(
            child.origin().y + child.size().height <= out.size().height,
            "child overflows the container's border box",
        );
    }

    #[test]
    fn content_box_default_keeps_authored_content_extent() {
        // The CSS default (`content-box`) is unchanged: an explicit `height: 52`
        // with `padding: 8` treats 52 as the CONTENT extent, so under a loose
        // constraint the border box grows to `52 + 16 = 68` (no subtraction).
        let style = LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            height: Dimension::Px(52.0),
            padding: Insets::uniform(8.0),
            // box_sizing defaults to ContentBox.
            ..LayoutStyle::default()
        };
        let children = [fixed_basis(1, 96.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut FixedChildMeasurer::new(vec![Size::new(96.0, 28.0)]),
        );
        // Content extent 52 + border/padding 16 = 68.
        approx(out.size().height, 68.0);
    }

    /// A wrap container of `width: 100` with three 40-wide items breaks into two
    /// lines (2 items then 1); the second line sits below the first (cross offset
    /// = line-1 height + gap) and the container cross size is the sum of the two
    /// line heights.
    #[test]
    fn flex_wrap_breaks_row_into_two_lines() {
        let style = LayoutStyle {
            width: Dimension::Px(100.0),
            flex_wrap: FlexWrap::Wrap,
            ..container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            )
        };
        let children = [
            fixed_basis(1, 40.0),
            fixed_basis(2, 40.0),
            fixed_basis(3, 40.0),
        ];
        // Each item measures 30px tall on the cross axis.
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Line 1: items 0 and 1 side by side at y = 0.
        approx(b[0].origin().x, 0.0);
        approx(b[0].origin().y, 0.0);
        approx(b[1].origin().x, 40.0);
        approx(b[1].origin().y, 0.0);
        // Line 2: item 2 below line 1 (cross offset = line-1 height 30 + gap 0).
        approx(b[2].origin().x, 0.0);
        approx(b[2].origin().y, 30.0);
        // Item cross sizes are the measured 30.
        approx(b[0].size().height, 30.0);
        approx(b[2].size().height, 30.0);
        // Container grew on the (auto) cross axis to the sum of the two line
        // heights: 30 + 30 = 60.
        approx(out.size().height, 60.0);
        approx(out.size().width, 100.0);
    }

    /// `flex-wrap: nowrap` (the default) keeps all three items on one line, packed
    /// against the main axis (overflowing the container), byte-identical to the
    /// historical single-line path.
    #[test]
    fn nowrap_keeps_all_items_on_one_line() {
        let style = LayoutStyle {
            width: Dimension::Px(100.0),
            flex_wrap: FlexWrap::NoWrap,
            ..container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            )
        };
        let children = [
            fixed_basis(1, 40.0),
            fixed_basis(2, 40.0),
            fixed_basis(3, 40.0),
        ];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // All three share y = 0, laid end to end (40, 80) — no wrapping.
        approx(b[0].origin().y, 0.0);
        approx(b[1].origin().y, 0.0);
        approx(b[2].origin().y, 0.0);
        approx(b[0].origin().x, 0.0);
        approx(b[1].origin().x, 40.0);
        approx(b[2].origin().x, 80.0);
    }

    /// An item wider than the container occupies its own flex line.
    #[test]
    fn wrap_puts_oversized_item_on_its_own_line() {
        let style = LayoutStyle {
            width: Dimension::Px(100.0),
            flex_wrap: FlexWrap::Wrap,
            ..container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            )
        };
        // Item 1 (150px) is wider than the 100px container.
        let children = [fixed_basis(1, 40.0), fixed_basis(2, 150.0)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(40.0, 30.0), Size::new(150.0, 30.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Item 0 on line 1 at y = 0; the oversized item 1 is pushed to its own
        // line 2 (below line 1's 30px height).
        approx(b[0].origin().y, 0.0);
        approx(b[1].origin().y, 30.0);
        approx(b[1].size().width, 150.0);
    }

    /// `flex-wrap: wrap-reverse` stacks the lines in reverse cross-axis order: the
    /// last line leads at the cross start.
    #[test]
    fn wrap_reverse_reverses_line_order() {
        let style = LayoutStyle {
            width: Dimension::Px(100.0),
            flex_wrap: FlexWrap::WrapReverse,
            ..container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            )
        };
        let children = [
            fixed_basis(1, 40.0),
            fixed_basis(2, 40.0),
            fixed_basis(3, 40.0),
        ];
        let mut measurer = FixedChildMeasurer::new(vec![
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
            Size::new(40.0, 30.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Reversed: the line holding item 2 leads at y = 0; items 0 and 1 follow
        // on the second-placed line at y = 30.
        approx(b[2].origin().y, 0.0);
        approx(b[0].origin().y, 30.0);
        approx(b[1].origin().y, 30.0);
    }

    #[test]
    fn column_auto_height_hugs_content_under_loose_constraint() {
        // A flex COLUMN with no explicit height, measured under a LOOSE
        // (indefinite) height, sizes its main axis to the stacked children + gaps
        // (shrink-to-fit) — not the full available height. This is the fix for a
        // nested flex-column item over-reporting its height when a parent flex row
        // cross-measures it (the wireframe "cramping").
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            8.0,
        );
        let children = [fixed_basis(1, 40.0), fixed_basis(2, 14.0)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 1000.0)),
            &mut FixedChildMeasurer::default(),
        );
        // 40 + gap 8 + 14 = 62, NOT the 1000px available height.
        approx(out.size().height, 62.0);
        // Under a TIGHT (definite) height the column fills as before: unchanged.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(300.0, 1000.0)),
            &mut FixedChildMeasurer::default(),
        );
        approx(out.size().height, 1000.0);
    }

    #[test]
    fn row_auto_height_hugs_tallest_item_under_loose_constraint() {
        // A flex ROW with no explicit height, measured under a LOOSE height, hugs
        // its tallest item's cross size instead of filling the available height.
        let children = [fixed_basis(1, 50.0), fixed_basis(2, 50.0)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 40.0), Size::new(50.0, 24.0)]);
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::FlexStart,
                0.0,
            ),
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        // Tallest item is 40, so the row hugs 40 — not the 1000px available.
        approx(out.size().height, 40.0);

        // With `align-items: stretch` the auto row height still hugs the tallest
        // item's HYPOTHETICAL (pre-stretch) cross (40), exercising the extra
        // measurement path; items themselves stretch to fill on the definite pass.
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(50.0, 40.0), Size::new(50.0, 24.0)]);
        let out = FlexLayout.layout(
            &container(
                FlexDirection::Row,
                JustifyContent::FlexStart,
                AlignItems::Stretch,
                0.0,
            ),
            &children,
            BoxConstraints::loose(Size::new(200.0, 1000.0)),
            &mut measurer,
        );
        approx(out.size().height, 40.0);
    }

    /// §9.7 `Σfactors < 1` clamp: a lone item with `flex-grow: 0.5` in a container
    /// with free space `F` grows by exactly `F · 0.5`, leaving the line half-empty.
    /// The old single-pass split gave the sole grow item ALL of `F` (wrong).
    #[test]
    fn lone_flex_grow_half_consumes_half_free_space() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 100.0);
        a.style.flex_grow = 0.5;
        let children = [a];
        // Container main = 200, base = 100, free = 100. grow 0.5 → +50, not +100.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        approx(out.child_boxes()[0].size().width, 150.0);
    }

    /// §9.7 min/max freeze + redistribute (grow): three equal-grow items where one
    /// caps at a `max-width` smaller than its share. The capped item freezes at its
    /// max and the OTHERS absorb the freed space, so the line still exactly fills.
    /// The old single-pass clamped once and left the line under-filled.
    #[test]
    fn grow_max_clamp_redistributes_to_others() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mk = |raw| {
            let mut c = fixed_basis(raw, 0.0);
            c.style.flex_grow = 1.0;
            c
        };
        let mut a = mk(1);
        a.style.max_width = Dimension::Px(50.0);
        let children = [a, mk(2), mk(3)];
        // Container = 300, bases 0, even share = 100. Item0 caps at 50; its 50 of
        // freed space is redistributed 1:1 to items 1 & 2 → 125 each. Sum = 300.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(300.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let b = out.child_boxes();
        approx(b[0].size().width, 50.0);
        approx(b[1].size().width, 125.0);
        approx(b[2].size().width, 125.0);
        // Line exactly fills: last item ends at the container main extent.
        approx(b[2].origin().x + b[2].size().width, 300.0);
    }

    /// §9.7 min/max freeze + redistribute (shrink): three equal-shrink items where
    /// one hits a `min-width` larger than its shrunk share. It freezes at its min
    /// and the others take up the remaining overflow, so the line exactly fills.
    #[test]
    fn shrink_min_clamp_redistributes_to_others() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mk = |raw| {
            let mut c = fixed_basis(raw, 50.0);
            c.style.flex_shrink = 1.0;
            c
        };
        let mut a = mk(1);
        a.style.min_width = Dimension::Px(45.0);
        let children = [a, mk(2), mk(3)];
        // Container = 100, bases 50 (Σ=150, overflow 50). Even shrink → 33.33 each,
        // but item0 floors at min 45. It freezes at 45; the remaining 45 of the
        // container is split by scaled shrink between items 1 & 2 → 27.5 each.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut FixedChildMeasurer::default(),
        );
        let b = out.child_boxes();
        approx(b[0].size().width, 45.0);
        approx(b[1].size().width, 27.5);
        approx(b[2].size().width, 27.5);
        approx(b[2].origin().x + b[2].size().width, 100.0);
    }

    /// A basic `flex: 1 1 auto` three-item even split (no clamping, factors ≥ 1) is
    /// unchanged by the freeze loop: the free space splits evenly.
    #[test]
    fn even_split_three_items_unchanged() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mk = |raw| {
            let mut c = fixed_basis(raw, 50.0);
            c.style.flex_grow = 1.0;
            c.style.flex_shrink = 1.0;
            c
        };
        let children = [mk(1), mk(2), mk(3)];
        // Container = 300, bases 50 (Σ=150), free = 150, +50 each → 100 each.
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(300.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        let b = out.child_boxes();
        approx(b[0].size().width, 100.0);
        approx(b[1].size().width, 100.0);
        approx(b[2].size().width, 100.0);
        approx(b[0].origin().x, 0.0);
        approx(b[1].origin().x, 100.0);
        approx(b[2].origin().x, 200.0);
    }

    /// A default flex item (auto flex-basis, `flex-shrink: 1`, `min: auto`) whose
    /// content size is supplied by the measurer.
    fn auto_item(raw: u64) -> LayoutChild {
        LayoutChild::new(
            NodeId::from_raw(raw),
            LayoutStyle {
                display: LayoutDisplay::Flex,
                ..LayoutStyle::default()
            },
        )
    }

    /// §4.5 automatic minimum size: a flex COLUMN of auto-basis items in a
    /// container SHORTER than their stacked content keeps each item at its
    /// content-based minimum (its measured content height) instead of shrinking it
    /// toward `0`; the content overflows the container. Without §4.5 the default
    /// `flex-shrink: 1` items would collapse below their content.
    #[test]
    fn column_auto_min_keeps_content_size_under_shrink() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let children = [auto_item(1), auto_item(2)];
        // Each item's content is 60 × 40; the container main (height) is only 50,
        // shorter than the stacked 40 + 40 = 80.
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(60.0, 40.0), Size::new(60.0, 40.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Each item keeps its 40px content height (floored by §4.5), NOT shrunk.
        approx(b[0].size().height, 40.0);
        approx(b[1].size().height, 40.0);
        // The second item sits below the first — the content overflows the 50px box.
        approx(b[0].origin().y, 0.0);
        approx(b[1].origin().y, 40.0);
    }

    /// An EXPLICIT `min-height: 0` opts out of §4.5: the automatic minimum applies
    /// only to an `auto`/unset main-axis min. With `min-height: 0` the items shrink
    /// below their content and share the undersized container exactly (25 each),
    /// proving the auto-min path is gated on `auto` and does not affect an explicit
    /// min.
    #[test]
    fn explicit_min_zero_still_shrinks_below_content() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let mk = |raw| {
            LayoutChild::new(
                NodeId::from_raw(raw),
                LayoutStyle {
                    display: LayoutDisplay::Flex,
                    min_height: Dimension::Px(0.0),
                    ..LayoutStyle::default()
                },
            )
        };
        let children = [mk(1), mk(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(60.0, 40.0), Size::new(60.0, 40.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // base 40 each, overflow 30 shared evenly → 25 each; the line exactly fills.
        approx(b[0].size().height, 25.0);
        approx(b[1].size().height, 25.0);
        approx(b[1].origin().y, 25.0);
    }

    /// §9.2 flex base = max-content for an auto-basis item, protected by §4.5: a
    /// bare max-content item beside a fill-available sibling in a row where the two
    /// together overflow keeps its FULL max-content width instead of being shrunk
    /// below its content (the select-header "One" → "On" clip). The fill sibling
    /// absorbs the shrink; the content item does not clip.
    #[test]
    fn auto_basis_item_keeps_max_content_beside_fill_sibling() {
        let style = container(
            FlexDirection::Row,
            JustifyContent::SpaceBetween,
            AlignItems::Center,
            10.0,
        );
        let children = [auto_item(1), auto_item(2)];
        // Item 0 is a 28-wide content item (the label); item 1 has an unbounded
        // natural width, so under a bounded row it FILLS the row and reports an
        // unbounded max-content (a flex/flow container with auto width — the
        // disclosure arrow), creating shrink pressure just like `.select-header`.
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(28.0, 14.0), Size::new(f32::INFINITY, 14.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 40.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // The content item keeps its full 28px max-content width — no clip. The fill
        // sibling (no automatic floor, since its max-content is unbounded) absorbs
        // ALL the shrink, so the line fits the 200px row exactly instead of
        // overflowing: 28 + gap 10 + sibling 162 = 200.
        approx(b[0].size().width, 28.0);
        approx(b[0].origin().x, 0.0);
        approx(b[1].size().width, 162.0);
        assert!(
            b[1].size().width < 200.0,
            "the fill sibling shrinks to make room for the content item (no overflow)",
        );
    }

    #[test]
    fn explicit_child_cross_size_wins_over_stretch() {
        // `align-items: stretch` normally fills the cross axis, but an author-set
        // cross size on the child overrides the stretch.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let mut a = fixed_basis(1, 50.0);
        a.style.height = Dimension::Px(30.0);
        let children = [a];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(200.0, 100.0)),
            &mut FixedChildMeasurer::default(),
        );
        // Cross (height) is the explicit 30, not the stretched 100.
        approx(out.child_boxes()[0].size().height, 30.0);
    }

    /// A [`ChildMeasurer`] that returns fixed natural sizes (clamped into the
    /// requested constraints, like [`FixedChildMeasurer`]) and TALLIES every
    /// [`measure_child`](ChildMeasurer::measure_child) call, so a test can prove the
    /// §4.5 content measure runs only when it can change the result.
    struct CountingMeasurer {
        sizes: Vec<Size>,
        calls: usize,
    }

    impl CountingMeasurer {
        fn new(sizes: Vec<Size>) -> Self {
            Self { sizes, calls: 0 }
        }
    }

    impl crate::algorithm::ChildMeasurer for CountingMeasurer {
        fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size {
            self.calls += 1;
            constraints.constrain(self.sizes.get(index).copied().unwrap_or(Size::ZERO))
        }
    }

    /// Perf regression guard: in the common column case (a tall-enough container
    /// whose auto-basis items are NOT clamped, and no shrink pressure) the §4.5
    /// content floor is derived from each item's flex base with ZERO extra measures.
    /// Only the per-item BASE measure runs — one call per item, not two.
    #[test]
    fn column_auto_min_takes_no_extra_measure_when_not_clamped() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [auto_item(1), auto_item(2), auto_item(3)];
        // Each item is 60 × 40; the container main (height) is 500 — comfortably
        // taller than the stacked 120, so no item is clamped and the line grows.
        let mut measurer = CountingMeasurer::new(vec![
            Size::new(60.0, 40.0),
            Size::new(60.0, 40.0),
            Size::new(60.0, 40.0),
        ]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 500.0)),
            &mut measurer,
        );
        // Geometry is unchanged: items keep their 40px content height, stacked.
        let b = out.child_boxes();
        approx(b[0].size().height, 40.0);
        approx(b[2].origin().y, 80.0);
        // Exactly ONE measure per item (the flex-base measure) — the §4.5
        // content-floor measure is elided (pre-fix this was TWO per item).
        assert_eq!(
            measurer.calls, 3,
            "no extra §4.5 content measure in the common case"
        );
    }

    /// The card-collapse §4.5 fix is preserved with its extra measure taken ONLY for
    /// the clamped item: a column item taller than an undersized container keeps its
    /// natural (max-content) height, and exactly one extra content measure is spent
    /// to discover it.
    #[test]
    fn column_auto_min_measures_only_the_clamped_item() {
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [auto_item(1), auto_item(2)];
        // Item 0 is 60 × 80 — TALLER than the 50px container main, so its flex base
        // clamps to 50 and the natural 80 must be measured (card collapse). Item 1
        // is 60 × 20 — fits, so it is NOT measured for a floor.
        let mut measurer =
            CountingMeasurer::new(vec![Size::new(60.0, 80.0), Size::new(60.0, 20.0)]);
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(100.0, 50.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Item 0 keeps its natural 80 (floored by §4.5 above the clamped base);
        // item 1 keeps its 20. Content overflows the 50px box.
        approx(b[0].size().height, 80.0);
        approx(b[1].size().height, 20.0);
        approx(b[1].origin().y, 80.0);
        // Two base measures + one content measure for the single clamped item = 3
        // (NOT 4 — the unclamped item spends no content measure).
        assert_eq!(
            measurer.calls, 3,
            "only the clamped item spends a §4.5 measure"
        );
    }

    // ---- intrinsic flex-basis / main size (audit Tier-1 §4-D) --------------

    /// An item that behaves like a paragraph: it fills the inline space it is
    /// offered, but never below `min` (its longest unbreakable word) and never
    /// above `max` (its whole content on one line). Unlike
    /// [`FixedChildMeasurer`] it ANSWERS the offer, which is the only way a probe
    /// can be caught not probing.
    struct Paragraph {
        min: f32,
        max: f32,
    }

    impl crate::algorithm::ChildMeasurer for Paragraph {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            Size::new(constraints.max.width.clamp(self.min, self.max), 10.0)
        }
    }

    /// A row of one item with the given main-axis style, in a `line` wide
    /// container: the item's arranged width. Neither grow nor shrink, so what
    /// comes back IS the flex base size.
    fn base_width_of(main: LayoutStyle, line: f32, measurer: &mut dyn ChildMeasurer) -> f32 {
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let children = [LayoutChild::new(NodeId::from_raw(1), main)];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(line, 50.0)),
            measurer,
        );
        out.child_boxes()[0].size().width
    }

    /// An item with the given `flex-basis`, rigid on both flex factors.
    fn rigid_basis(basis: Dimension) -> LayoutStyle {
        LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_basis: basis,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..LayoutStyle::default()
        }
    }

    #[test]
    fn flex_basis_max_content_measures_rather_than_degrading_to_auto() {
        // A 100px row holding an item whose content wants 200 on one line.
        // `flex-basis: max-content` is 200 and overflows.
        //
        // `auto` is 200 as well, and that is the point of §9.2.3 step E: an auto
        // basis with no definite main size resolves to the item's max-content
        // size, not to the space it was offered. This line used to assert `100` —
        // the offer — and that assertion was the defect: a rigid item that took
        // the whole line left nothing for the siblings that had asked to grow.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        approx(
            base_width_of(rigid_basis(Dimension::MaxContent), 100.0, &mut m),
            200.0,
        );
        approx(
            base_width_of(rigid_basis(Dimension::Auto), 100.0, &mut m),
            200.0,
        );
    }

    #[test]
    fn flex_basis_min_content_measures_rather_than_degrading_to_auto() {
        // The same 100px row: `min-content` is the 30-wide unbreakable run, so the
        // item shrinks to its widest word instead of filling the line.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        approx(
            base_width_of(rigid_basis(Dimension::MinContent), 100.0, &mut m),
            30.0,
        );
    }

    #[test]
    fn an_intrinsic_width_feeds_the_flex_base_when_flex_basis_is_auto() {
        // §9.2 step 3: with `flex-basis: auto` the item's main-axis SIZE supplies
        // the base — including when it is a keyword, which used to fall through to
        // the natural-size measure.
        let mut m = Paragraph {
            min: 30.0,
            max: 200.0,
        };
        let style = LayoutStyle {
            display: LayoutDisplay::Flex,
            width: Dimension::MinContent,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..LayoutStyle::default()
        };
        approx(base_width_of(style, 100.0, &mut m), 30.0);
    }

    #[test]
    fn fit_content_basis_shrinks_a_fill_available_item_to_its_content() {
        // The item is a nested `width: auto` block: it fills whatever it is
        // offered, and reports its own 200-wide content only when the offer is
        // unbounded. `fit-content` = min(max-content 200, max(min-content 0,
        // stretch 400)) = 200 — the content.
        //
        // `auto` is 200 too. A flex item is content-sized along the main axis
        // unless it asks to grow; filling the 400px row — which this line used to
        // assert — is what a *block* child does, and answering the block question
        // here is what starved growing siblings.
        struct FillAvailable;
        impl crate::algorithm::ChildMeasurer for FillAvailable {
            fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
                let width = if constraints.max.width.is_finite() {
                    constraints.max.width
                } else {
                    200.0
                };
                Size::new(width, 10.0)
            }
        }
        approx(
            base_width_of(
                rigid_basis(Dimension::FitContent),
                400.0,
                &mut FillAvailable,
            ),
            200.0,
        );
        approx(
            base_width_of(rigid_basis(Dimension::Auto), 400.0, &mut FillAvailable),
            200.0,
        );
    }

    /// An **unbounded** main axis with a growing item must terminate, and the
    /// item must take its flex base size.
    ///
    /// This is the shape that hung the whole application. An unbounded axis is
    /// the `max-content` *question*, so there is no free space to distribute; the
    /// old loop distributed it anyway, growing the item toward `inf`, clamping it
    /// against an `inf` maximum, and computing the violation as `inf − inf` —
    /// **NaN**. NaN is false against `== 0`, `> 0` and `< 0` alike, so the sign
    /// test that decides what to freeze froze nothing, the next pass recomputed
    /// the identical state, and the frame never returned. The event loop went
    /// with it: the window stopped drawing and could not even be closed.
    ///
    /// It took a squeezed window to reach: a container whose padding exceeds the
    /// space it is given has no room left, and the block axis is then offered
    /// unbounded.
    #[test]
    fn an_unbounded_main_axis_terminates_and_keeps_the_base_size() {
        struct Rigid;
        impl crate::algorithm::ChildMeasurer for Rigid {
            fn measure_child(&mut self, _index: usize, _constraints: BoxConstraints) -> Size {
                Size::new(30.0, 10.0)
            }
        }
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let item = LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_grow: 8.0,
            flex_basis: Dimension::Px(30.0),
            ..LayoutStyle::default()
        };
        let children = [LayoutChild::new(NodeId::from_raw(1), item)];

        // The assertion is that this returns at all; the value pins the meaning.
        let out = FlexLayout.layout(
            &style,
            &children,
            // A test's own container constraints, not a measure of a child: no
            // style in this fixture authors a percentage, so there is nothing for
            // a basis to change. Written out longhand only because
            // `BoxConstraints::loose` cannot express a half-infinite `max`.
            BoxConstraints {
                min: Size::ZERO,
                max: Size::new(f32::INFINITY, 50.0),
                percent_basis: None,
            },
            &mut Rigid,
        );
        approx(out.child_boxes()[0].size().width, 30.0);
    }

    /// The same for the block axis, which is the one a squeezed window actually
    /// leaves unbounded.
    #[test]
    fn an_unbounded_cross_of_a_column_terminates_too() {
        struct Rigid;
        impl crate::algorithm::ChildMeasurer for Rigid {
            fn measure_child(&mut self, _index: usize, _constraints: BoxConstraints) -> Size {
                Size::new(10.0, 30.0)
            }
        }
        let style = container(
            FlexDirection::Column,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let item = LayoutStyle {
            display: LayoutDisplay::Flex,
            flex_grow: 8.0,
            flex_basis: Dimension::Px(30.0),
            ..LayoutStyle::default()
        };
        let children = [LayoutChild::new(NodeId::from_raw(1), item)];
        let out = FlexLayout.layout(
            &style,
            &children,
            // As above: the fixture's own constraints, no percentage in sight.
            BoxConstraints {
                min: Size::ZERO,
                max: Size::new(50.0, f32::INFINITY),
                percent_basis: None,
            },
            &mut Rigid,
        );
        approx(out.child_boxes()[0].size().height, 30.0);
    }

    #[test]
    fn an_intrinsic_basis_costs_no_measure_for_an_item_that_authored_none() {
        // The guard on the cost of the extra probes: an ordinary auto-basis item
        // in a stretching row (which needs no cross measure either) still spends
        // exactly one measure, so the keywords are paid for only by the items that
        // author them.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::Stretch,
            0.0,
        );
        let children = [auto_item(1), auto_item(2)];
        let mut measurer =
            CountingMeasurer::new(vec![Size::new(30.0, 10.0), Size::new(30.0, 10.0)]);
        FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(400.0, 50.0)),
            &mut measurer,
        );
        assert_eq!(measurer.calls, 2, "one base measure per item, no probes");
    }

    /// An item that is itself a **wrapping flex row** of two rigid 60x60 boxes,
    /// laid out under exactly the constraints the outer container hands it.
    ///
    /// The one thing [`FixedChildMeasurer`] cannot do and this must: couple the
    /// item's block size to its inline size. Two 60-wide boxes fit on one line at
    /// 200 (height 60) and wrap at 100 (height 120), so the height that comes back
    /// says out loud which inline size the item believed it had.
    struct WrappingRow {
        /// The item's own style, re-laid-out on every measure.
        style: LayoutStyle,
    }

    impl crate::algorithm::ChildMeasurer for WrappingRow {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            let box_style = LayoutStyle {
                display: LayoutDisplay::Flex,
                width: Dimension::Px(60.0),
                height: Dimension::Px(60.0),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..LayoutStyle::default()
            };
            let kids = [
                LayoutChild::new(NodeId::from_raw(11), box_style.clone()),
                LayoutChild::new(NodeId::from_raw(12), box_style),
            ];
            let mut inner =
                FixedChildMeasurer::new(vec![Size::new(60.0, 60.0), Size::new(60.0, 60.0)]);
            FlexLayout
                .layout(&self.style, &kids, constraints, &mut inner)
                .size()
        }
    }

    #[test]
    fn a_percentage_item_is_measured_against_the_container_not_against_itself() {
        // A 400-wide row, one `width: 50%` item, `align-items: flex-start` so the
        // item's cross size is MEASURED rather than stretched.
        let style = container(
            FlexDirection::Row,
            JustifyContent::FlexStart,
            AlignItems::FlexStart,
            0.0,
        );
        let item_style = LayoutStyle {
            display: LayoutDisplay::Flex,
            width: Dimension::Percent(50.0),
            flex_wrap: FlexWrap::Wrap,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..LayoutStyle::default()
        };
        let children = [LayoutChild::new(NodeId::from_raw(1), item_style.clone())];
        let out = FlexLayout.layout(
            &style,
            &children,
            BoxConstraints::tight(Size::new(400.0, 200.0)),
            &mut WrappingRow { style: item_style },
        );
        let item = out.child_boxes()[0].size();
        approx(item.width, 200.0);
        // 60 = one line. Before the fix this was 120: the cross measure handed the
        // item a tight main axis and no basis, so the item re-resolved its own
        // `width: 50%` against its own 200 and laid its content out at 100.
        approx(item.height, 60.0);
    }
}
