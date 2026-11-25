#![doc = include_str!("../../docs/api/algorithm/flex_line_placement.doc/_module.md")]

use obscur_geometry::{Point, Rect, Size};

use super::content_distribution::content_distribution;
use crate::model::{AlignItems, LayoutBox, LayoutStyle};

use super::flex_axis_bounds::clamp_cross;
use super::flex_cross_size::cross_item_size;
pub(super) use super::flex_line_args::LineArgs;
use super::flex_main_margins::{reserved, used};
use super::flex_main_probe::content_max_main;
use super::placed_line::PlacedLine;
use super::resolve_flexible_lengths::resolve_flexible_lengths;

#[doc = include_str!("../../docs/api/algorithm/flex_line_placement.doc/item_align.md")]
fn item_align(item: &LayoutStyle, container: &LayoutStyle) -> AlignItems {
    item.align_self.resolve(container.align_items)
}

#[doc = include_str!("../../docs/api/algorithm/flex_line_placement.doc/place_line.md")]
pub(super) fn place_line(args: LineArgs) -> PlacedLine {
    let LineArgs {
        line,
        base_main,
        flex_base,
        main_min,
        main_max,
        deferred_floor,
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
        line_cross_stretch,
        percent_basis,
        measurer,
        child_boxes,
    } = args;

    let count = line.len();
    let gap_span = gap * (count.saturating_sub(1) as f32);

    // Margins consume main-axis space alongside the gaps; both are constants that
    // do not flex, so the justification pass below reuses this sum.
    let sum_margin_main: f32 = line
        .iter()
        .map(|&i| reserved(&main_lead[i]) + reserved(&main_trail[i]))
        .sum();

    // §4.5 content floors for the DEFERRED items (row main axis / definite basis)
    // are `content.min(fb) <= fb`, so they can only bite when this line's items are
    // actually shrunk below their flex base — i.e. when the base main sizes overflow
    // the space available to them. Resolve them lazily HERE, and only then: a line
    // that grows or exactly fits takes ZERO extra measures (its floors would never
    // bind, so the pre-§4.5 baseline of 0 is output-identical).
    let available = line_main - gap_span - sum_margin_main;
    let sum_base: f32 = line.iter().map(|&i| base_main[i]).sum();
    let main_min_owned;
    let main_min: &[f32] = if sum_base > available && line.iter().any(|&i| deferred_floor[i]) {
        let mut resolved = main_min.to_vec();
        for &i in line {
            if deferred_floor[i] {
                let content = content_max_main(&mut *measurer, i, is_row, container_cross);
                let floor = if content.is_finite() {
                    content.min(flex_base[i])
                } else {
                    0.0
                };
                resolved[i] = floor.min(main_max[i]).max(0.0);
            }
        }
        main_min_owned = resolved;
        &main_min_owned
    } else {
        main_min
    };

    // Steps 3–4: resolve the final main sizes with the canonical
    // "resolve flexible lengths" freeze loop (§9.7).
    let final_main = resolve_flexible_lengths(
        line,
        flex_base,
        base_main,
        main_min,
        main_max,
        children,
        line_main,
        gap_span + sum_margin_main,
    );

    // Step 5: cross size per item, and the line's cross extent.
    //
    // For the single-line path (`stretch_target = Some`), a stretched item fills
    // the container cross exactly as before and no measurement is taken. For the
    // multi-line path, every item's cross size is resolved (explicit) or measured,
    // the line cross is their max, and a stretched item fills that line cross.
    let mut cross_size: Vec<f32> = Vec::with_capacity(count);
    let line_cross;
    let mut content_cross = 0.0_f32;
    if let Some(target) = stretch_target {
        for (pos, &i) in line.iter().enumerate() {
            let cross = cross_item_size(
                item_align(&children[i].style, style) == AlignItems::Stretch,
                target,
                &children[i].style,
                is_row,
                final_main[pos],
                container_cross,
                percent_basis,
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
                    percent_basis,
                    i,
                    &mut *measurer,
                );
                content_cross = content_cross.max(hypothetical);
            }
        }
        line_cross = target;
    } else {
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
                percent_basis,
                i,
                &mut *measurer,
            );
            base_cross.push(bc);
        }
        let natural_lc = base_cross.iter().copied().fold(0.0_f32, f32::max);
        // `align-content: stretch` grows the line to its allotted cross size;
        // the target is never smaller than the natural cross (free space is
        // non-negative), and `None` leaves the natural sizing untouched.
        let lc = line_cross_stretch.map_or(natural_lc, |t| t.max(natural_lc));
        for (pos, &i) in line.iter().enumerate() {
            let cross_style = &children[i].style;
            let cross_dim = if is_row {
                cross_style.height.clone()
            } else {
                cross_style.width.clone()
            };
            let has_explicit = cross_dim.resolve(container_cross).is_some();
            let cross = if item_align(cross_style, style) == AlignItems::Stretch && !has_explicit {
                // A stretch fills the line cross, but must still stop at the
                // item's own cross max (§9.4 step 11) — `base_cross` came from
                // `cross_item_size` already clamped, this path did not.
                clamp_cross(lc, cross_style, is_row, container_cross)
            } else {
                base_cross[pos]
            };
            cross_size.push(cross);
        }
        line_cross = lc;
        content_cross = lc;
    }

    // The alignment reference is the container cross for the single-line path and
    // the line cross for the multi-line path.
    let align_ref = match stretch_target {
        Some(_) => container_cross,
        None => line_cross,
    };

    // Step 6: distribute the line's free space — to `auto` margins first, and to
    // `justify_content` only if they leave any (CSS Flexbox §8.1).
    let total_final: f32 = final_main.iter().copied().sum();
    let occupied_main = total_final + gap_span + sum_margin_main;
    let positioning_main = if line_main.is_finite() {
        line_main
    } else {
        occupied_main.max(0.0)
    };
    let remaining = positioning_main - occupied_main;

    // §8.1: "if the remaining free space is positive and at least one main-axis
    // margin on this line is `auto`, the free space is distributed equally among
    // those margins" — and, crucially, `justify-content` then has nothing left to
    // distribute. That ordering is the whole behaviour: `margin-left: auto` on one
    // item pushes it (and everything after it) to the end of the row *regardless*
    // of `justify-content`, which is exactly why the idiom is used.
    let auto_main: usize = line
        .iter()
        .map(|&i| usize::from(main_lead[i].is_auto()) + usize::from(main_trail[i].is_auto()))
        .sum();
    // Each `auto` edge's share. Zero when there are none, so `absorbed` below is a
    // no-op and every existing line places byte-identically.
    let auto_share = if auto_main > 0 && remaining > 0.0 {
        remaining / auto_main as f32
    } else {
        0.0
    };
    // Auto margins absorb positive space only. On overflow they resolve to
    // zero and positional justification still receives the negative remainder.
    let free_for_justify = if auto_share > 0.0 { 0.0 } else { remaining };
    let (leading, spacing) = content_distribution(count, free_for_justify, style.justify_content);

    // Steps 7 & 8: position each item and map back to (x, y).
    let mut cursor = leading;
    for pos in 0..count {
        let i = line[pos];
        // A leading main-axis margin pushes the item further along the main axis
        // before it is placed — by its own length, or by its share of the line's
        // free space when it is `auto`.
        cursor += used(&main_lead[i], auto_share);
        let main = final_main[pos];
        let cross = cross_size[pos];
        let cross_start = cross_offset
            + reserved(&cross_lead[i])
            + match item_align(&children[i].style, style) {
                AlignItems::FlexStart
                | AlignItems::Start
                | AlignItems::Stretch
                | AlignItems::Baseline => 0.0,
                AlignItems::FlexEnd | AlignItems::End | AlignItems::LastBaseline => {
                    align_ref - cross
                }
                AlignItems::Center => (align_ref - cross) / 2.0,
            };
        let main_start = if is_reverse {
            positioning_main - cursor - main
        } else {
            cursor
        };
        let (origin, size) = if is_row {
            (
                Point::new(bp_left + main_start, bp_top + cross_start),
                Size::new(main, cross),
            )
        } else {
            (
                Point::new(bp_left + cross_start, bp_top + main_start),
                Size::new(cross, main),
            )
        };
        child_boxes[i] = LayoutBox::new(Rect::new(origin, size));
        cursor += main + used(&main_trail[i], auto_share) + gap + spacing;
    }

    PlacedLine {
        cross: line_cross,
        content_cross,
    }
}
