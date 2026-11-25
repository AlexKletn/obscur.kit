//! Stretching a paragraph's eligible lines to fill the wrap width
//! (`text-align: justify`, spec §5.10).

use std::collections::HashSet;

use super::LineBreak;
use super::packed_line::PackedLine;

#[doc = include_str!("../../docs/api/layout/line_justification.doc/justify_lines.md")]
pub(super) fn justify_lines(lines: &mut [PackedLine], breaks: &[LineBreak], max_width: f32) {
    let Some(last) = lines.len().checked_sub(1) else {
        return;
    };
    // Byte offsets that begin a word (every break opportunity); a glyph whose
    // cluster equals one starts a new word, so the glyph before it ends a gap.
    let word_starts: HashSet<usize> = breaks.iter().map(LineBreak::offset).collect();
    for (index, line) in lines.iter_mut().enumerate() {
        if index == last || line.mandatory_break_after || line.word_count < 2 {
            continue;
        }
        justify_line(line, &word_starts, max_width);
    }
}

#[doc = include_str!("../../docs/api/layout/line_justification.doc/justify_line.md")]
fn justify_line(line: &mut PackedLine, word_starts: &HashSet<usize>, max_width: f32) {
    let extra = max_width - line.width();
    if extra <= 0.0 {
        return;
    }
    let glyphs = &mut line.glyphs;
    // Trailing-glyph index of every interior gap (before a word-start glyph).
    let gap_ends: Vec<usize> = (1..glyphs.len())
        .filter(|&j| word_starts.contains(&glyphs[j].glyph.cluster()))
        .map(|j| j - 1)
        .collect();
    if gap_ends.is_empty() {
        return;
    }
    #[allow(clippy::cast_precision_loss)]
    let add = extra / gap_ends.len() as f32;
    for index in gap_ends {
        let glyph = glyphs[index].glyph;
        glyphs[index].glyph = glyph.with_x_advance(glyph.x_advance() + add);
    }
}
