#![doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_text::layout::{ParagraphLayout, ParagraphLine};
use obscur_text::selection::CaretAffinity;
use obscur_text::shaping::TextDirection;
use obscur_tree::tree::RetainedTree;

use super::paragraph_shaping::{ParagraphShaping, font_size_of};
use super::{ComputedStyles, LayoutBoxes};

#[doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/boundary_after_forced_break.md")]
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

/// The line index a caret byte `index` sits on within a wrapped `layout`
/// (spec §5.10): the first line whose byte range **ends past** `index`, else
/// the last line — so a caret at a soft-wrap/`\n` boundary sits at the start of
/// the *next* line and a caret at the text end sits on the final line. A
/// single-line layout always resolves to line `0`.
pub(crate) fn line_of_byte(layout: &ParagraphLayout, index: usize) -> usize {
    let lines = layout.lines();
    lines
        .iter()
        .position(|line| index < line.range().end)
        .unwrap_or_else(|| lines.len().saturating_sub(1))
}

#[doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/line_start_byte.md")]
fn line_start_byte(layout: &ParagraphLayout, li: usize) -> usize {
    let lines = layout.lines();
    match lines.get(li) {
        Some(line) if line.glyph_count() > 0 => line.range().start,
        Some(_) if li > 0 => lines[li - 1].range().end,
        _ => 0,
    }
}

// Visit source cluster boundaries in paint order, including both edges at
// a bidi transition. A cluster may contain several glyphs with one byte offset.
fn visit_line_clusters(
    line: &ParagraphLine,
    mut visit: impl FnMut(usize, usize, f32, f32) -> bool,
) {
    let mut pen = 0.0;
    for segment in line.segments() {
        let run = segment.run();
        let mut clusters = run
            .glyphs()
            .chunk_by(|a, b| a.cluster() == b.cluster())
            .peekable();
        let rtl = run.direction() == TextDirection::RightToLeft;
        let mut previous = None;
        while let Some(glyphs) = clusters.next() {
            let start = glyphs[0].cluster();
            let end = if rtl {
                previous.unwrap_or(run.source().end)
            } else {
                clusters
                    .peek()
                    .map_or(run.source().end, |next| next[0].cluster())
            };
            let (left, right) = if rtl { (end, start) } else { (start, end) };
            let next_pen = pen + glyphs.iter().map(|glyph| glyph.x_advance()).sum::<f32>();
            if !visit(left, right, pen, next_pen) {
                return;
            }
            pen = next_pen;
            previous = Some(start);
        }
    }
}

/// Returns the caret's x within the line, before its alignment offset. Glyph
/// clusters are visited in paint order with the same advances used for rendering.
/// An RTL cluster's logical start is at its right edge. At a bidi boundary the
/// caret uses the leading edge of the cluster at the requested byte, with the
/// preceding cluster's trailing edge as the fallback.
pub(crate) fn line_caret_x(line: &ParagraphLine, index: usize) -> f32 {
    let mut trailing_edge = (None, 0.0);
    let mut leading_x = None;
    visit_line_clusters(line, |left, right, from, to| {
        let (start, leading, trailing) = if left <= right {
            (left, from, to)
        } else {
            (right, to, from)
        };
        if index == start {
            leading_x = Some(leading);
            return false;
        }
        if index > start && trailing_edge.0.is_none_or(|previous| start > previous) {
            trailing_edge = (Some(start), trailing);
        }
        true
    });
    leading_x.unwrap_or(trailing_edge.1)
}

/// Maps x within a line to a byte offset using the nearest painted cluster edge.
/// The midpoint resolves to the left edge. Clicks beyond the text use the
/// outermost visual edge, whose byte offset depends on the run's direction.
fn byte_at_line_local_x(line: &ParagraphLine, local_x: f32) -> usize {
    let mut byte = line.range().end;
    visit_line_clusters(line, |left, right, from, to| {
        if local_x <= from + (to - from) / 2.0 {
            byte = left;
            return false;
        }
        byte = right;
        local_x > to
    });
    byte
}

#[doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/caret_line_geometry.md")]
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

#[doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/caret_line_geometry_affinity.md")]
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

#[doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/caret_offset_at_point.md")]
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
    let layout = shaping.shape_editable_lines(tree, computed, boxes, node, content, box_width);
    let lines = layout.lines();
    if lines.is_empty() {
        return 0;
    }
    // The line whose box contains `local_y` (top ≤ y < top+height), else the
    // last line (a click below the text lands on the final line).
    let li = lines
        .iter()
        .position(|line| local_y < line.top() + line.height())
        .unwrap_or(lines.len() - 1);
    byte_at_line_x(&layout, li, content, local_x)
}

#[doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/byte_at_line_x.md")]
fn byte_at_line_x(layout: &ParagraphLayout, li: usize, content: &str, target_x: f32) -> usize {
    let Some(line) = layout.lines().get(li) else {
        return 0;
    };
    let line_start = line_start_byte(layout, li);
    // Undo the line's alignment offset, then resolve against this line's own
    // spaced/tabbed advances (the exact inverse of the painted caret x).
    let byte = byte_at_line_local_x(line, target_x - line.x_offset());
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

#[doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/caret_line_move.md")]
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
    let layout = shaping.shape_editable_lines(tree, computed, boxes, node, content, box_width);
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
    Some(byte_at_line_x(&layout, target, content, target_x))
}

#[doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/caret_visual_line_bound.md")]
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
    let layout = shaping.shape_editable_lines(tree, computed, boxes, node, content, box_width);
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

#[doc = include_str!("../../docs/api/orchestration/caret_geometry.doc/caret_line_scroll_geometry.md")]
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
    let layout = shaping.shape_editable_lines(tree, computed, boxes, node, content, box_width);
    let (top, height, _x_offset, _x) = caret_line_geometry(shaping, &layout, font_size, index);
    // The scrollable content height is the bottom-most wrapped line's edge
    // (never less than the caret's own line, so a single line is exact).
    let content_height = layout
        .lines()
        .last()
        .map_or(top + height, |line| line.top() + line.height());
    (top, height, content_height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_text::font::FontId;
    use obscur_text::layout::{TextAlignment, TextOverflow, WhiteSpace};
    use obscur_text::shaping::{GlyphRun, ShapedGlyph, ShapedRun};
    use obscur_text::span::TextRange;

    #[test]
    fn wrapped_caret_follows_visual_rtl_cluster_edges() {
        let range = TextRange::new(0, 6);
        let run = GlyphRun::new(
            vec![
                ShapedGlyph::new(1, 4, 3.0),
                ShapedGlyph::new(2, 4, 7.0),
                ShapedGlyph::new(3, 2, 10.0),
                ShapedGlyph::new(4, 0, 10.0),
            ],
            TextDirection::RightToLeft,
            12.0,
            range,
        );
        let layout = ParagraphLayout::wrap(
            &[ShapedRun::new(FontId::new(0), run).with_level(1)],
            "אבג",
            30.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Visible,
            WhiteSpace::Normal,
        );
        let line = &layout.lines()[0];
        assert_eq!(line.range(), range);
        assert_eq!(line_caret_x(line, 0), 30.0);
        assert_eq!(line_caret_x(line, 2), 20.0);
        assert_eq!(line_caret_x(line, 4), 10.0);
        assert_eq!(line_caret_x(line, 6), 0.0);
        assert_eq!(byte_at_line_local_x(line, -1.0), 6);
        assert_eq!(byte_at_line_local_x(line, 4.0), 6);
        assert_eq!(byte_at_line_local_x(line, 7.0), 4);
        assert_eq!(byte_at_line_local_x(line, 16.0), 2);
        assert_eq!(byte_at_line_local_x(line, 31.0), 0);
    }
}
