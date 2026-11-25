//! One line's shaped segments, in visual order (spec §5.10, UAX #9 L2).

use crate::shaping::{GlyphRun, ShapedRun, TextDirection};
use crate::span::TextRange;

use super::tagged_glyph::TaggedGlyph;

#[doc = include_str!("../../docs/api/layout/line_segments.doc/line_segments.md")]
pub(super) fn line_segments(glyphs: &[TaggedGlyph], line_end: usize) -> Vec<ShapedRun> {
    let mut segments = Vec::new();
    let mut index = 0;
    while index < glyphs.len() {
        let font = glyphs[index].font;
        let direction = glyphs[index].direction;
        let font_size = glyphs[index].font_size;
        let level = glyphs[index].level;
        let synth_oblique = glyphs[index].synth_oblique;
        let synth_bold = glyphs[index].synth_bold;
        let seg_start = glyphs[index].glyph.cluster();

        let mut run_glyphs = Vec::new();
        while index < glyphs.len()
            && glyphs[index].font == font
            && glyphs[index].direction == direction
            && glyphs[index].level == level
            && glyphs[index].font_size == font_size
            && glyphs[index].synth_oblique == synth_oblique
            && glyphs[index].synth_bold == synth_bold
        {
            run_glyphs.push(glyphs[index].glyph);
            index += 1;
        }

        // The segment ends where the next segment begins (or the line end).
        let seg_end = glyphs.get(index).map_or(line_end, |t| t.glyph.cluster());
        let range = TextRange::new(seg_start.min(seg_end), seg_start.max(seg_end));
        if direction == TextDirection::RightToLeft {
            run_glyphs.reverse();
            for cluster in run_glyphs.chunk_by_mut(|a, b| a.cluster() == b.cluster()) {
                cluster.reverse();
            }
        }
        let run = GlyphRun::new(run_glyphs, direction, font_size, range);
        segments.push(
            ShapedRun::new(font, run)
                .with_level(level)
                .with_synthesis(synth_oblique, synth_bold),
        );
    }
    reorder_visual(&mut segments);
    segments
}

#[doc = include_str!("../../docs/api/layout/line_segments.doc/reorder_visual.md")]
fn reorder_visual(segments: &mut [ShapedRun]) {
    let Some(max_level) = segments.iter().map(ShapedRun::level).max() else {
        return;
    };
    let lowest_odd = segments
        .iter()
        .map(ShapedRun::level)
        .filter(|lvl| lvl % 2 == 1)
        .min();
    let Some(lowest_odd) = lowest_odd else {
        // No odd level: pure LTR line, nothing to reorder.
        return;
    };

    let mut lvl = max_level;
    while lvl >= lowest_odd {
        let mut start = 0;
        while start < segments.len() {
            if segments[start].level() >= lvl {
                let mut end = start;
                while end < segments.len() && segments[end].level() >= lvl {
                    end += 1;
                }
                segments[start..end].reverse();
                start = end;
            } else {
                start += 1;
            }
        }
        lvl -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::font::FontId;
    use crate::shaping::{ShapedGlyph, TextDirection};

    /// A one-glyph segment in `font` at bidi embedding `level`.
    fn segment(font: u32, level: u8) -> ShapedRun {
        ShapedRun::new(
            FontId::new(font),
            GlyphRun::new(
                vec![ShapedGlyph::new(1, 0, 6.0)],
                TextDirection::LeftToRight,
                10.0,
                TextRange::new(0, 1),
            ),
        )
        .with_level(level)
    }

    /// The fonts of `segments`, in the order they sit in.
    fn fonts(segments: &[ShapedRun]) -> Vec<u32> {
        segments.iter().map(|s| s.font().get()).collect()
    }

    #[test]
    fn an_all_even_line_is_left_untouched() {
        let mut segments = vec![segment(0, 0), segment(1, 0), segment(2, 2)];
        reorder_visual(&mut segments);
        assert_eq!(fonts(&segments), vec![0, 1, 2]);
    }

    #[test]
    fn a_run_of_odd_level_segments_is_reversed() {
        let mut segments = vec![segment(0, 1), segment(1, 1)];
        reorder_visual(&mut segments);
        assert_eq!(fonts(&segments), vec![1, 0]);
    }

    #[test]
    fn a_lone_odd_segment_keeps_its_place_between_even_neighbours() {
        let mut segments = vec![segment(0, 0), segment(1, 1), segment(2, 0)];
        reorder_visual(&mut segments);
        assert_eq!(fonts(&segments), vec![0, 1, 2]);
    }

    #[test]
    fn a_higher_level_span_reorders_within_the_lower_one() {
        // Levels [0,1,2,1,0]: L2 reverses the level-2 span (a no-op for one
        // segment), then the level-1 span 1..4 → fonts [0, 3, 2, 1, 4].
        let mut segments = vec![
            segment(0, 0),
            segment(1, 1),
            segment(2, 2),
            segment(3, 1),
            segment(4, 0),
        ];
        reorder_visual(&mut segments);
        assert_eq!(fonts(&segments), vec![0, 3, 2, 1, 4]);
    }

    #[test]
    fn segment_splits_on_font_size_and_synthesis_boundaries() {
        // CSS Inline Layout 3 §3.1 / CSS Text 3 §1.3: within one line and one
        // font face, glyphs that differ in used font size or in synthesized
        // bold/oblique (CSS Fonts 4 §2.4, spec §5.18) form distinct inline
        // boxes and MUST NOT be coalesced into a single run rendered at the
        // first glyph's size / synthesis. The coordinator builds such a line by
        // concatenating two independently-styled shaped spans of the same font
        // and bidi level before wrapping:
        //   [font 0, level 0, size 20, synth_bold=true,  "AB"] (clusters 0,1)
        //   [font 0, level 0, size 12, synth_bold=false, "cd"] (clusters 2,3)
        // → TWO segments (20px faux-bold, then 12px plain), not one 20px bold.
        let tag = |cluster: usize, font_size: f32, synth_bold: bool| TaggedGlyph {
            font: FontId::new(0),
            direction: TextDirection::LeftToRight,
            font_size,
            level: 0,
            synth_oblique: false,
            synth_bold,
            glyph: ShapedGlyph::new(1, cluster, 6.0),
        };
        let glyphs = vec![
            tag(0, 20.0, true),
            tag(1, 20.0, true),
            tag(2, 12.0, false),
            tag(3, 12.0, false),
        ];
        let segments = line_segments(&glyphs, 4);
        assert_eq!(segments.len(), 2);
        // First segment: 20px, faux-bold, both "AB" glyphs — NOT the whole line.
        assert_eq!(segments[0].run().font_size(), 20.0);
        assert!(segments[0].synth_bold());
        assert!(!segments[0].synth_oblique());
        assert_eq!(segments[0].run().len(), 2);
        // Second segment: 12px, not faux-bold, both "cd" glyphs.
        assert_eq!(segments[1].run().font_size(), 12.0);
        assert!(!segments[1].synth_bold());
        assert_eq!(segments[1].run().len(), 2);
        // Both share font 0 and level 0: the split is purely on size/synthesis.
        assert_eq!(segments[0].font(), FontId::new(0));
        assert_eq!(segments[1].font(), FontId::new(0));
        assert_eq!(segments[0].level(), 0);
        assert_eq!(segments[1].level(), 0);
    }

    #[test]
    fn an_empty_line_has_no_segments_to_reorder() {
        let mut segments: Vec<ShapedRun> = Vec::new();
        reorder_visual(&mut segments);
        assert!(segments.is_empty());
    }
}
