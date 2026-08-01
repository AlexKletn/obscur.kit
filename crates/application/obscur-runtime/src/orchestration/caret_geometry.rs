#![doc = include_str!("caret_geometry.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_style::computed::ComputedStyle;
use obscur_text::font::{FontAttributes, FontFallback};
use obscur_text::layout::{ParagraphLayout, ParagraphLine};
use obscur_text::selection::CaretAffinity;
use obscur_text::shaping::TextDirection;
use obscur_tree::tree::RetainedTree;

use super::paragraph_shaping::{ParagraphShaping, font_size_of, font_size_of_style};
use super::{ComputedStyles, LayoutBoxes, attributes_for, direction_for, families_for};

#[doc = include_str!("caret_geometry.doc/boundary_after_forced_break.md")]
fn boundary_after_forced_break(content: &str, index: usize) -> bool {
    let index = index.min(content.len());
    if !content.is_char_boundary(index) {
        return false;
    }
    content[..index].chars().next_back().is_some_and(|c| {
        matches!(
            c,
            '\n' | '\r' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
        )
    })
}

#[doc = include_str!("caret_geometry.doc/caret_offset_x.md")]
// The shaping inputs mirror `shape_lines`/`shape_hyphen`: cohesive positional
// arguments of a leaf helper, `attributes` by reference to match the
// `shape_paragraph` convention this pass uses throughout.
#[allow(clippy::too_many_arguments, clippy::trivially_copy_pass_by_ref)]
pub(super) fn caret_offset_x(
    shaping: &ParagraphShaping,
    content: &str,
    fallback: &FontFallback,
    attributes: &FontAttributes,
    direction: TextDirection,
    font_size: f32,
    index: usize,
) -> f32 {
    shaping
        .paragraph()
        .shape_paragraph(
            content,
            fallback,
            attributes,
            shaping.default_face(),
            font_size,
            direction,
        )
        .iter()
        .flat_map(|run| run.run().glyphs().iter().copied())
        .filter(|glyph| glyph.cluster() < index)
        .map(|glyph| glyph.x_advance())
        .sum()
}

#[doc = include_str!("caret_geometry.doc/caret_offset_at_x.md")]
#[must_use]
pub(crate) fn caret_offset_at_x(
    shaping: &ParagraphShaping,
    content: &str,
    style: Option<&ComputedStyle>,
    local_x: f32,
) -> usize {
    let fallback = families_for(style);
    let attributes = attributes_for(style);
    let direction = direction_for(style);
    let font_size = font_size_of_style(style);
    let mut pen = 0.0_f32;
    for glyph in shaping
        .paragraph()
        .shape_paragraph(
            content,
            &fallback,
            &attributes,
            shaping.default_face(),
            font_size,
            direction,
        )
        .iter()
        .flat_map(|run| run.run().glyphs().iter().copied())
    {
        let advance = glyph.x_advance();
        // The boundary just left of this glyph sits at `pen`; cross to the next
        // boundary only once `local_x` is past this glyph's midpoint.
        if local_x <= pen + advance / 2.0 {
            return glyph.cluster();
        }
        pen += advance;
    }
    // Past every glyph: the caret lands at the end of the text.
    content.len()
}

#[doc = include_str!("caret_geometry.doc/caret_advance_x.md")]
#[must_use]
pub(crate) fn caret_advance_x(
    shaping: &ParagraphShaping,
    content: &str,
    style: Option<&ComputedStyle>,
    index: usize,
) -> f32 {
    let fallback = families_for(style);
    let attributes = attributes_for(style);
    let direction = direction_for(style);
    let font_size = font_size_of_style(style);
    caret_offset_x(
        shaping,
        content,
        &fallback,
        &attributes,
        direction,
        font_size,
        index,
    )
}

#[doc = include_str!("caret_geometry.doc/line_of_byte.md")]
fn line_of_byte(layout: &ParagraphLayout, index: usize) -> usize {
    let lines = layout.lines();
    lines
        .iter()
        .position(|line| index < line.range().end)
        .unwrap_or_else(|| lines.len().saturating_sub(1))
}

#[doc = include_str!("caret_geometry.doc/line_start_byte.md")]
fn line_start_byte(layout: &ParagraphLayout, li: usize) -> usize {
    let lines = layout.lines();
    match lines.get(li) {
        Some(line) if line.glyph_count() > 0 => line.range().start,
        Some(_) if li > 0 => lines[li - 1].range().end,
        _ => 0,
    }
}

#[doc = include_str!("caret_geometry.doc/line_caret_x.md")]
pub(super) fn line_caret_x(line: &ParagraphLine, index: usize) -> f32 {
    line.segments()
        .iter()
        .flat_map(|segment| segment.run().glyphs().iter().copied())
        .filter(|glyph| glyph.cluster() < index)
        .map(|glyph| glyph.x_advance())
        .sum()
}

#[doc = include_str!("caret_geometry.doc/byte_at_line_local_x.md")]
fn byte_at_line_local_x(line: &ParagraphLine, line_end: usize, local_x: f32) -> usize {
    let mut pen = 0.0_f32;
    for glyph in line
        .segments()
        .iter()
        .flat_map(|segment| segment.run().glyphs().iter())
    {
        let advance = glyph.x_advance();
        // The boundary just left of this glyph sits at `pen`; cross to the next
        // boundary only once `local_x` is past this glyph's midpoint.
        if local_x <= pen + advance / 2.0 {
            return glyph.cluster();
        }
        pen += advance;
    }
    // Past every glyph on this line: clamp to the line's end byte.
    line_end
}

#[doc = include_str!("caret_geometry.doc/caret_line_geometry.md")]
pub(super) fn caret_line_geometry(
    shaping: &ParagraphShaping,
    layout: &ParagraphLayout,
    font_size: f32,
    index: usize,
) -> (f32, f32, f32, f32) {
    let li = line_of_byte(layout, index);
    let Some(line) = layout.lines().get(li) else {
        return (
            0.0,
            shaping.paragraph().line_metrics(font_size).line_height(),
            0.0,
            0.0,
        );
    };
    (
        line.top(),
        line.height(),
        line.x_offset(),
        line_caret_x(line, index),
    )
}

#[doc = include_str!("caret_geometry.doc/caret_line_geometry_affinity.md")]
pub(super) fn caret_line_geometry_affinity(
    shaping: &ParagraphShaping,
    layout: &ParagraphLayout,
    content: &str,
    font_size: f32,
    index: usize,
    affinity: CaretAffinity,
) -> (f32, f32, f32, f32) {
    let lines = layout.lines();
    let downstream = line_of_byte(layout, index);
    if affinity.is_upstream()
        && downstream > 0
        && lines[downstream - 1].range().end == index
        && line_start_byte(layout, downstream) == index
        && !boundary_after_forced_break(content, index)
    {
        let line = &lines[downstream - 1];
        return (
            line.top(),
            line.height(),
            line.x_offset(),
            line_caret_x(line, index),
        );
    }
    caret_line_geometry(shaping, layout, font_size, index)
}

#[doc = include_str!("caret_geometry.doc/caret_offset_at_point.md")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn caret_offset_at_point(
    shaping: &ParagraphShaping,
    tree: &RetainedTree,
    computed: &ComputedStyles,
    boxes: &LayoutBoxes,
    node: NodeId,
    content: &str,
    box_width: f32,
    local_x: f32,
    local_y: f32,
) -> usize {
    let style = computed.get(&node);
    let layout = shaping.shape_text_lines(tree, computed, boxes, node, content, box_width);
    let lines = layout.lines();
    if lines.is_empty() {
        return caret_offset_at_x(shaping, content, style, local_x);
    }
    // The line whose box contains `local_y` (top ≤ y < top+height), else the
    // last line (a click below the text lands on the final line).
    let li = lines
        .iter()
        .position(|line| local_y < line.top() + line.height())
        .unwrap_or(lines.len() - 1);
    byte_at_line_x(shaping, &layout, li, content, style, local_x)
}

#[doc = include_str!("caret_geometry.doc/byte_at_line_x.md")]
fn byte_at_line_x(
    shaping: &ParagraphShaping,
    layout: &ParagraphLayout,
    li: usize,
    content: &str,
    style: Option<&ComputedStyle>,
    target_x: f32,
) -> usize {
    let Some(line) = layout.lines().get(li) else {
        return caret_offset_at_x(shaping, content, style, target_x);
    };
    let line_start = line_start_byte(layout, li);
    // Undo the line's alignment offset, then resolve against this line's own
    // spaced/tabbed advances (the exact inverse of the painted caret x).
    let byte = byte_at_line_local_x(line, line.range().end, target_x - line.x_offset());
    // The maximum caret byte that stays on this line: the line's end, minus a
    // trailing `\n` (whose caret belongs before it, on this line).
    let end = line.range().end.max(line_start).min(content.len());
    let line_max = if content
        .get(line_start..end)
        .is_some_and(|s| s.ends_with('\n'))
    {
        end - 1
    } else {
        end
    };
    byte.clamp(line_start, line_max)
}

#[doc = include_str!("caret_geometry.doc/caret_line_move.md")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn caret_line_move(
    shaping: &ParagraphShaping,
    tree: &RetainedTree,
    computed: &ComputedStyles,
    boxes: &LayoutBoxes,
    node: NodeId,
    content: &str,
    box_width: f32,
    index: usize,
    down: bool,
) -> Option<usize> {
    let style = computed.get(&node);
    let layout = shaping.shape_text_lines(tree, computed, boxes, node, content, box_width);
    let cur = line_of_byte(&layout, index);
    let target = if down {
        let next = cur + 1;
        (next < layout.lines().len()).then_some(next)?
    } else {
        cur.checked_sub(1)?
    };
    // The caret's current pen x within its line (over the line's own
    // spaced/tabbed advances), re-expressed from the text box origin by adding
    // the TARGET line's alignment offset so `byte_at_line_x` maps it back
    // consistently (it undoes that same offset).
    let within_line = layout
        .lines()
        .get(cur)
        .map_or(0.0, |line| line_caret_x(line, index));
    let target_x = layout.lines()[target].x_offset() + within_line;
    Some(byte_at_line_x(
        shaping, &layout, target, content, style, target_x,
    ))
}

#[doc = include_str!("caret_geometry.doc/caret_visual_line_bound.md")]
#[allow(clippy::too_many_arguments)]
#[must_use]
pub(crate) fn caret_visual_line_bound(
    shaping: &ParagraphShaping,
    tree: &RetainedTree,
    computed: &ComputedStyles,
    boxes: &LayoutBoxes,
    node: NodeId,
    content: &str,
    box_width: f32,
    index: usize,
    to_end: bool,
) -> usize {
    let layout = shaping.shape_text_lines(tree, computed, boxes, node, content, box_width);
    let li = line_of_byte(&layout, index);
    if to_end {
        layout
            .lines()
            .get(li)
            .map_or(content.len(), |line| line.range().end)
    } else {
        line_start_byte(&layout, li)
    }
}

#[doc = include_str!("caret_geometry.doc/caret_line_scroll_geometry.md")]
#[must_use]
pub(crate) fn caret_line_scroll_geometry(
    shaping: &ParagraphShaping,
    tree: &RetainedTree,
    computed: &ComputedStyles,
    boxes: &LayoutBoxes,
    node: NodeId,
    content: &str,
    box_width: f32,
    index: usize,
) -> (f32, f32, f32) {
    let font_size = font_size_of(computed, node);
    let layout = shaping.shape_text_lines(tree, computed, boxes, node, content, box_width);
    let (top, height, _x_offset, _x) = caret_line_geometry(shaping, &layout, font_size, index);
    // The scrollable content height is the bottom-most wrapped line's edge
    // (never less than the caret's own line, so a single line is exact).
    let content_height = layout
        .lines()
        .last()
        .map_or(top + height, |line| line.top() + line.height());
    (top, height, content_height)
}
