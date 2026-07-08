//! `FlexLayout`: single-line flex layout for the `Flex` formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{AlignItems, Dimension, JustifyContent, LayoutBox, LayoutStyle};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// Clamps a main-axis size into the item's `[min, max]` range, resolving each
/// bound against the container main extent (spec §4.6).
///
/// A [`Dimension::Auto`] bound is absent and imposes no limit; `max` is applied
/// first, then `min`, so `min` wins on conflict (CSS `min-width` overrides
/// `max-width`). The result is never negative. Both bounds default to `Auto`, so
/// an item with neither set is returned unchanged.
fn clamp_main(value: f32, min: Dimension, max: Dimension, container_main: f32) -> f32 {
    let mut clamped = value;
    if let Some(max_v) = max.resolve(container_main) {
        clamped = clamped.min(max_v);
    }
    if let Some(min_v) = min.resolve(container_main) {
        clamped = clamped.max(min_v);
    }
    clamped.max(0.0)
}

/// The item's main-axis min / max dimensions for the container's flow direction:
/// `width` bounds along a row, `height` bounds along a column.
fn main_bounds(style: &LayoutStyle, is_row: bool) -> (Dimension, Dimension) {
    if is_row {
        (style.min_width, style.max_width)
    } else {
        (style.min_height, style.max_height)
    }
}

/// The item's total border + padding along the given axis (`horizontal` for the
/// row main / column cross axis, else vertical).
///
/// Used to expand an explicit `content-box` size into the border-box extent the
/// item actually occupies, so a `width: 0` / `height: 0` item with a border keeps
/// its border widths instead of collapsing to a zero-extent box.
fn axis_bp(style: &LayoutStyle, horizontal: bool) -> f32 {
    if horizontal {
        style.border.left + style.border.right + style.padding.left + style.padding.right
    } else {
        style.border.top + style.border.bottom + style.padding.top + style.padding.bottom
    }
}

/// Single-line (no-wrap) flex layout (spec §4.1, §4.2, §4.6).
///
/// Lays items out along a main axis chosen by `style.flex_direction`, resolves
/// their main sizes by distributing free space via `flex_grow` / `flex_shrink`,
/// positions them with `justify_content` along the main axis, and aligns them
/// with `align_items` along the cross axis.
///
/// # Algorithm (Phase-1 subset)
///
/// 1. The main axis is horizontal for [`Row`]/[`RowReverse`] and vertical for
///    [`Column`]/[`ColumnReverse`]. Container content extents honour the
///    container's own explicit `width` / `height` (resolved against
///    `constraints.max`); an `Auto` extent falls back to `constraints.max` minus
///    border + padding. Both are clamped to be non-negative. Free space, `fr`
///    growth, percentages, and `justify_content` all resolve against this extent.
/// 2. Each item's **base main size** is `flex_basis` (resolved against the
///    container main extent); else its main-axis size (`width` for a row,
///    `height` for a column); else its measured natural main size (measured with
///    loose constraints up to the container content box). Never negative.
/// 3. `free = container_main - (sum of base sizes) - gap * (count - 1)`.
/// 4. If `free > 0` and `sum(flex_grow) > 0`, each item grows by
///    `free * grow_i / sum(grow)`. If `free < 0` and the scaled shrink factor
///    `sum(flex_shrink_i * base_i) > 0`, each item shrinks proportionally to
///    `flex_shrink_i * base_i`, clamped so no final main size is negative.
///    Otherwise base sizes are kept and the slack is handled by justification.
/// 5. Cross size per item is the container cross extent when
///    `align_items == Stretch` (unless the item sets an explicit cross-axis size,
///    which wins over the stretch), else the item's measured cross size (measured
///    with its final main size fixed and a loose cross axis).
/// 6. `justify_content` turns the leftover main-axis space into a leading offset
///    and an inter-item spacing (`FlexStart`, `FlexEnd`, `Center`,
///    `SpaceBetween`, `SpaceAround`, `SpaceEvenly`); when nothing is left over
///    the leading offset and spacing are zero.
/// 7. Each item is positioned along the main axis (leading + accumulated sizes +
///    gaps + justify spacing) and along the cross axis per `align_items`
///    (`FlexStart` → 0, `FlexEnd` → `container_cross - item_cross`, `Center` →
///    half, `Stretch` → 0 filling the cross axis). Reverse directions place items
///    in reversed order along the main axis.
/// 8. Main/cross coordinates map back to x/y and are offset into the content box.
///    The node border-box size is the container content size plus border and
///    padding, then clamped into `constraints`.
///
/// [`Row`]: crate::model::FlexDirection::Row
/// [`RowReverse`]: crate::model::FlexDirection::RowReverse
/// [`Column`]: crate::model::FlexDirection::Column
/// [`ColumnReverse`]: crate::model::FlexDirection::ColumnReverse
///
/// # Out of scope for Phase 1
///
/// This is deliberately a **subset**: no wrapping / multi-line flex, no `order`
/// reordering, no baseline alignment (`AlignItems::Baseline` is not modelled),
/// and no `align-self` per-item cross override. These belong to later phases.
///
/// Per-item **main-axis min/max clamping** (`min-width`/`max-width` in a row,
/// the height bounds in a column) and **margins** (which consume main-axis space
/// and offset each item) *are* honoured; cross-axis min/max and margin
/// interaction with `align-items`/auto margins remain later work.
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
        let is_row = style.flex_direction.is_row();
        let is_reverse = style.flex_direction.is_reverse();
        let gap = style.gap;

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
        let content_width = style
            .width
            .resolve(constraints.max.width)
            .map_or(constraints.max.width - bp_x, |w| {
                style.box_sizing.content_extent(w, bp_x)
            })
            .max(0.0);
        let content_height = style
            .height
            .resolve(constraints.max.height)
            .map_or(constraints.max.height - bp_y, |h| {
                style.box_sizing.content_extent(h, bp_y)
            })
            .max(0.0);
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
            let m = child.style.margin;
            let (lead, trail) = if is_row {
                if is_reverse {
                    (m.right, m.left)
                } else {
                    (m.left, m.right)
                }
            } else if is_reverse {
                (m.bottom, m.top)
            } else {
                (m.top, m.bottom)
            };
            main_lead.push(lead);
            main_trail.push(trail);
            cross_lead.push(if is_row { m.top } else { m.left });
        }

        // Step 2: base main size per item, clamped into the item's main-axis
        // min/max range.
        let mut base_main = Vec::with_capacity(count);
        for (index, child) in children.iter().enumerate() {
            let main_dim = if is_row {
                child.style.width
            } else {
                child.style.height
            };
            // An explicit main size (from `flex-basis` or `width`/`height`) is
            // authored against the item's box-sizing, so it is expanded to the
            // border-box extent the item occupies (`content-box` adds the item's
            // main-axis border + padding). The measured fallback already reports a
            // border box, so it is used as-is.
            let base = child
                .style
                .flex_basis
                .resolve(container_main)
                .or_else(|| main_dim.resolve(container_main))
                .map_or_else(
                    || {
                        let measured = measurer.measure_child(
                            index,
                            BoxConstraints::loose(Size::new(content_width, content_height)),
                        );
                        if is_row {
                            measured.width
                        } else {
                            measured.height
                        }
                    },
                    |explicit| {
                        child
                            .style
                            .box_sizing
                            .border_extent(explicit, axis_bp(&child.style, is_row))
                    },
                );
            let (min_main, max_main) = main_bounds(&child.style, is_row);
            base_main.push(clamp_main(base.max(0.0), min_main, max_main, container_main));
        }

        // Steps 3–8: break the items into flex lines, then size / justify /
        // align each line independently and stack the lines along the cross axis.
        //
        // `flex-wrap: nowrap` (the default) has exactly one line holding every
        // item; it is byte-for-byte identical to the historical single-line path
        // (`stretch_target = Some(container_cross)` reproduces the old
        // fill-the-container stretch and container-relative alignment).
        let mut child_boxes = vec![LayoutBox::default(); count];

        if !style.flex_wrap.is_multi_line() {
            let line: Vec<usize> = (0..count).collect();

            // The main-axis extent the single line's items occupy: their base main
            // sizes, the inter-item gaps, and their main-axis margins. This is the
            // content main size a column hugs to under an indefinite height.
            let gap_span = gap * (count.saturating_sub(1) as f32);
            let sum_base: f32 = base_main.iter().sum();
            let sum_margin_main: f32 = main_lead
                .iter()
                .zip(main_trail.iter())
                .map(|(l, t)| l + t)
                .sum();
            let content_main = sum_base + gap_span + sum_margin_main;

            let placed = place_line(LineArgs {
                line: &line,
                base_main: &base_main,
                children,
                style,
                is_row,
                is_reverse,
                line_main: container_main,
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

            let border_box = Size::new(content_width + bp_x, final_content_height + bp_y);
            return LayoutOutput::new(constraints.constrain(border_box), child_boxes);
        }

        // Multi-line: greedily fill lines along the main axis. An item (with its
        // main-axis margins) that does not fit on the current non-empty line
        // starts a new one; an item wider than the container occupies its own line.
        let mut lines: Vec<Vec<usize>> = Vec::new();
        let mut current: Vec<usize> = Vec::new();
        let mut current_main = 0.0_f32;
        for index in 0..count {
            let item_main = base_main[index] + main_lead[index] + main_trail[index];
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

        // Stack each line at the running cross offset; the cross-axis gap sits
        // between adjacent lines. The container's cross extent grows to the sum of
        // the line cross sizes when it is `auto` (so all lines fit without
        // overlap); an explicit cross size is kept as authored.
        let mut cross_offset = 0.0_f32;
        let mut total_cross = 0.0_f32;
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                cross_offset += gap;
                total_cross += gap;
            }
            let line_cross = place_line(LineArgs {
                line,
                base_main: &base_main,
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
                // The multi-line path already resolves every item's own cross size
                // to compute the line cross, so no extra pre-stretch pass is needed.
                content_cross_needed: false,
                measurer: &mut *measurer,
                child_boxes: &mut child_boxes,
            })
            .cross;
            cross_offset += line_cross;
            total_cross += line_cross;
        }

        let cross_is_auto = if is_row {
            style.height.resolve(constraints.max.height).is_none()
        } else {
            style.width.resolve(constraints.max.width).is_none()
        };
        let (final_content_width, final_content_height) = if is_row {
            let h = if cross_is_auto { total_cross } else { content_height };
            (content_width, h)
        } else {
            let w = if cross_is_auto { total_cross } else { content_width };
            (w, content_height)
        };

        let border_box = Size::new(final_content_width + bp_x, final_content_height + bp_y);
        LayoutOutput::new(constraints.constrain(border_box), child_boxes)
    }
}

/// Inputs for [`place_line`]: one flex line's items plus the shared container /
/// style context. Grouped into a struct because the single-line sizing pass
/// legitimately needs this many parameters and a positional argument list would
/// be error-prone.
struct LineArgs<'a, 'm> {
    /// The item indices on this line, in source order.
    line: &'a [usize],
    /// Base main size per item, indexed by the global item index.
    base_main: &'a [f32],
    /// The flex items.
    children: &'a [LayoutChild],
    /// The flex container's style.
    style: &'a LayoutStyle,
    /// Whether the main axis is horizontal.
    is_row: bool,
    /// Whether items are placed in reverse main-axis order.
    is_reverse: bool,
    /// The main-axis extent this line's free space / justification resolve against.
    line_main: f32,
    /// Gap between adjacent items on the line.
    gap: f32,
    /// Leading / trailing main-axis margins, indexed by the global item index.
    main_lead: &'a [f32],
    main_trail: &'a [f32],
    /// Leading cross-axis margin, indexed by the global item index.
    cross_lead: &'a [f32],
    /// When `Some(target)`, `stretch` fills `target` and alignment resolves
    /// against `container_cross` (the single-line / no-wrap semantics). When
    /// `None`, the line's cross size is the max item cross size and `stretch`
    /// fills that (the multi-line semantics).
    stretch_target: Option<f32>,
    /// The container's cross extent (the measurement bound and, for the
    /// single-line path, the alignment reference).
    container_cross: f32,
    /// Content-box origin offsets.
    bp_left: f32,
    bp_top: f32,
    /// The line's leading cross-axis offset (0 for a single line).
    cross_offset: f32,
    /// Whether the caller needs [`PlacedLine::content_cross`] (the line's
    /// pre-stretch content cross extent). When `false`, a stretched item's
    /// hypothetical cross is not measured, keeping the definite path byte-identical
    /// and avoiding an unnecessary measurement.
    content_cross_needed: bool,
    /// Child measurer.
    measurer: &'m mut dyn ChildMeasurer,
    /// Output boxes, written at each item's global index.
    child_boxes: &'a mut [LayoutBox],
}

/// The result of placing one flex line.
struct PlacedLine {
    /// The line's cross extent, used to stack lines (the stretch target for the
    /// single-line path, the max item cross for the multi-line path).
    cross: f32,
    /// The line's **pre-stretch** content cross extent — the tallest item's
    /// hypothetical (measured or explicit) cross size. Equals [`Self::cross`] for
    /// the multi-line path; for the single-line path it is the height an
    /// (auto, indefinite) row hugs to. Only meaningful when the caller set
    /// `content_cross_needed`.
    content_cross: f32,
}

/// Sizes, justifies, and cross-aligns one flex line, writing each item's box and
/// returning the line's cross extent (spec §4.2, §4.6).
///
/// Distributes the line's main-axis free space via `flex_grow` / `flex_shrink`,
/// re-clamps into each item's main min/max range, positions items with
/// `justify_content` along the main axis, and aligns them with `align_items`
/// along the cross axis. Reverse directions place items in reversed main order.
fn place_line(args: LineArgs) -> PlacedLine {
    let LineArgs {
        line,
        base_main,
        children,
        style,
        is_row,
        is_reverse,
        line_main,
        gap,
        main_lead,
        main_trail,
        cross_lead,
        stretch_target,
        container_cross,
        bp_left,
        bp_top,
        cross_offset,
        content_cross_needed,
        measurer,
        child_boxes,
    } = args;

    let count = line.len();
    let gap_span = gap * (count.saturating_sub(1) as f32);

    // Step 3: free space along the main axis (margins consume space too).
    let sum_base: f32 = line.iter().map(|&i| base_main[i]).sum();
    let sum_margin_main: f32 = line.iter().map(|&i| main_lead[i] + main_trail[i]).sum();
    let free = line_main - sum_base - gap_span - sum_margin_main;

    // Step 4: resolve final main sizes via grow / shrink.
    let sum_grow: f32 = line.iter().map(|&i| children[i].style.flex_grow).sum();
    let mut final_main: Vec<f32> = line.iter().map(|&i| base_main[i]).collect();
    if free > 0.0 && sum_grow > 0.0 {
        for (final_size, &i) in final_main.iter_mut().zip(line.iter()) {
            *final_size += free * children[i].style.flex_grow / sum_grow;
        }
    } else if free < 0.0 {
        let sum_weight: f32 = line
            .iter()
            .map(|&i| children[i].style.flex_shrink * base_main[i])
            .sum();
        if sum_weight > 0.0 {
            for (final_size, &i) in final_main.iter_mut().zip(line.iter()) {
                let weight = children[i].style.flex_shrink * base_main[i];
                *final_size = (*final_size + free * weight / sum_weight).max(0.0);
            }
        }
    }

    // Re-clamp the grown/shrunk main sizes into each item's min/max range so a
    // `min-width` / `max-width` bound holds after free-space distribution.
    for (final_size, &i) in final_main.iter_mut().zip(line.iter()) {
        let (min_main, max_main) = main_bounds(&children[i].style, is_row);
        *final_size = clamp_main(*final_size, min_main, max_main, line_main);
    }

    // Step 5: cross size per item, and the line's cross extent.
    //
    // For the single-line path (`stretch_target = Some`), a stretched item fills
    // the container cross exactly as before and no measurement is taken. For the
    // multi-line path, every item's cross size is resolved (explicit) or measured,
    // the line cross is their max, and a stretched item fills that line cross.
    let mut cross_size: Vec<f32> = Vec::with_capacity(count);
    let line_cross;
    let mut content_cross = 0.0_f32;
    match stretch_target {
        Some(target) => {
            for (pos, &i) in line.iter().enumerate() {
                let cross = cross_item_size(
                    style.align_items == AlignItems::Stretch,
                    target,
                    &children[i].style,
                    is_row,
                    final_main[pos],
                    container_cross,
                    i,
                    &mut *measurer,
                );
                cross_size.push(cross);
                // The line's pre-stretch content cross: a stretched item does not
                // report its used (filled) cross but its hypothetical size — its
                // explicit cross, else its measured cross. Only measured when the
                // caller needs it (an auto, indefinite row), so the definite path
                // takes no extra measurement and is byte-identical.
                if content_cross_needed {
                    let hypothetical = cross_item_size(
                        false,
                        0.0,
                        &children[i].style,
                        is_row,
                        final_main[pos],
                        container_cross,
                        i,
                        &mut *measurer,
                    );
                    content_cross = content_cross.max(hypothetical);
                }
            }
            line_cross = target;
        }
        None => {
            // First resolve/measure each item's own cross size (no stretch), so
            // the line cross size is known before stretched items fill it.
            let mut base_cross: Vec<f32> = Vec::with_capacity(count);
            for (pos, &i) in line.iter().enumerate() {
                let bc = cross_item_size(
                    false,
                    0.0,
                    &children[i].style,
                    is_row,
                    final_main[pos],
                    container_cross,
                    i,
                    &mut *measurer,
                );
                base_cross.push(bc);
            }
            let lc = base_cross.iter().copied().fold(0.0_f32, f32::max);
            for (pos, &i) in line.iter().enumerate() {
                let cross_style = &children[i].style;
                let cross_dim = if is_row {
                    cross_style.height
                } else {
                    cross_style.width
                };
                let has_explicit = cross_dim.resolve(container_cross).is_some();
                let cross = if style.align_items == AlignItems::Stretch && !has_explicit {
                    lc
                } else {
                    base_cross[pos]
                };
                cross_size.push(cross);
            }
            line_cross = lc;
            content_cross = lc;
        }
    }

    // The alignment reference is the container cross for the single-line path and
    // the line cross for the multi-line path.
    let align_ref = match stretch_target {
        Some(_) => container_cross,
        None => line_cross,
    };

    // Step 6: justify_content → leading offset and inter-item spacing.
    let total_final: f32 = final_main.iter().copied().sum();
    let remaining = line_main - total_final - gap_span - sum_margin_main;
    let (leading, spacing) = if remaining <= 0.0 || count == 0 {
        (0.0, 0.0)
    } else {
        let n = count as f32;
        match style.justify_content {
            JustifyContent::FlexStart => (0.0, 0.0),
            JustifyContent::FlexEnd => (remaining, 0.0),
            JustifyContent::Center => (remaining / 2.0, 0.0),
            JustifyContent::SpaceBetween => {
                if count > 1 {
                    (0.0, remaining / (n - 1.0))
                } else {
                    (0.0, 0.0)
                }
            }
            JustifyContent::SpaceAround => {
                let s = remaining / n;
                (s / 2.0, s)
            }
            JustifyContent::SpaceEvenly => {
                let s = remaining / (n + 1.0);
                (s, s)
            }
        }
    };

    // Steps 7 & 8: position each item and map back to (x, y).
    let mut cursor = leading;
    let order: Vec<usize> = if is_reverse {
        (0..count).rev().collect()
    } else {
        (0..count).collect()
    };
    for &pos in &order {
        let i = line[pos];
        // A leading main-axis margin pushes the item further along the main axis
        // before it is placed.
        cursor += main_lead[i];
        let main = final_main[pos];
        let cross = cross_size[pos];
        let cross_start = cross_offset
            + cross_lead[i]
            + match style.align_items {
                AlignItems::FlexStart | AlignItems::Start | AlignItems::Stretch => 0.0,
                AlignItems::FlexEnd | AlignItems::End => align_ref - cross,
                AlignItems::Center => (align_ref - cross) / 2.0,
            };
        let (origin, size) = if is_row {
            (
                Point::new(bp_left + cursor, bp_top + cross_start),
                Size::new(main, cross),
            )
        } else {
            (
                Point::new(bp_left + cross_start, bp_top + cursor),
                Size::new(cross, main),
            )
        };
        child_boxes[i] = LayoutBox::new(Rect::new(origin, size));
        cursor += main + main_trail[i] + gap + spacing;
    }

    PlacedLine {
        cross: line_cross,
        content_cross,
    }
}

/// Resolves one item's cross-axis size.
///
/// When `stretch` is set, the item fills `stretch_target` unless it sets an
/// explicit cross-axis size (which wins, expanded to its border-box extent).
/// Otherwise the item's cross size is measured with its final main size fixed and
/// a loose cross axis bounded by `container_cross`.
#[allow(clippy::too_many_arguments)]
fn cross_item_size(
    stretch: bool,
    stretch_target: f32,
    cross_style: &LayoutStyle,
    is_row: bool,
    main: f32,
    container_cross: f32,
    index: usize,
    measurer: &mut dyn ChildMeasurer,
) -> f32 {
    if stretch {
        let cross_dim = if is_row {
            cross_style.height
        } else {
            cross_style.width
        };
        cross_dim
            .resolve(container_cross)
            .map_or(stretch_target, |explicit| {
                cross_style
                    .box_sizing
                    .border_extent(explicit, axis_bp(cross_style, !is_row))
            })
    } else {
        let item_constraints = if is_row {
            BoxConstraints {
                min: Size::new(main, 0.0),
                max: Size::new(main, container_cross),
            }
        } else {
            BoxConstraints {
                min: Size::new(0.0, main),
                max: Size::new(container_cross, main),
            }
        };
        let measured = measurer.measure_child(index, item_constraints);
        if is_row {
            measured.height
        } else {
            measured.width
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FlexLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{
        AlignItems, BoxSizing, Dimension, FlexDirection, FlexWrap, JustifyContent, LayoutDisplay,
        LayoutStyle,
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
        // Last child takes the leading slot; first child follows.
        approx(boxes[1].origin().x, 0.0);
        approx(boxes[0].origin().x, 50.0);
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
}
