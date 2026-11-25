//! One flattened, font-tagged glyph of a paragraph's glyph stream (spec §5.10).

use crate::font::FontId;
use crate::shaping::{ShapedGlyph, ShapedRun, TextDirection};

/// One flattened glyph tagged with the font, direction, font size, and bidi
/// embedding level of the [`ShapedRun`] it came from. Flattening the fallback
/// result into this stream lets word-splitting and greedy packing treat the
/// whole paragraph uniformly while retaining enough per-glyph provenance to
/// rebuild same-font, same-level segments and reorder them into visual order.
#[derive(Clone, Copy)]
pub(super) struct TaggedGlyph {
    pub(super) font: FontId,
    pub(super) direction: TextDirection,
    pub(super) font_size: f32,
    pub(super) level: u8,
    /// Whether the segment this glyph belongs to is faux-sheared (spec §5.18).
    pub(super) synth_oblique: bool,
    /// Whether the segment this glyph belongs to is faux-bolded (spec §5.18).
    pub(super) synth_bold: bool,
    pub(super) glyph: ShapedGlyph,
}

impl TaggedGlyph {
    #[doc = include_str!("../../docs/api/layout/tagged_glyph.doc/flatten.md")]
    pub(super) fn flatten(runs: &[ShapedRun]) -> Vec<Self> {
        let mut stream = Vec::new();
        for shaped in runs {
            let run = shaped.run();
            let font = shaped.font();
            let direction = run.direction();
            let font_size = run.font_size();
            let level = shaped.level();
            let synth_oblique = shaped.synth_oblique();
            let synth_bold = shaped.synth_bold();
            let mut append = |glyph| {
                stream.push(Self {
                    font,
                    direction,
                    font_size,
                    level,
                    synth_oblique,
                    synth_bold,
                    glyph,
                });
            };
            let glyphs = run.glyphs();
            if direction == TextDirection::RightToLeft {
                for cluster in glyphs.chunk_by(|a, b| a.cluster() == b.cluster()).rev() {
                    cluster.iter().copied().for_each(&mut append);
                }
            } else {
                glyphs.iter().copied().for_each(append);
            }
        }
        stream
    }

    #[doc = include_str!("../../docs/api/layout/tagged_glyph.doc/source_char.md")]
    pub(super) fn source_char(self, text: &str) -> Option<char> {
        text.get(self.glyph.cluster()..)
            .and_then(|rest| rest.chars().next())
    }

    /// Whether this glyph's source character is a CSS word separator
    /// (CSS Text 3 §8.2): U+0020 SPACE or U+00A0 NO-BREAK SPACE.
    pub(super) fn is_word_separator(self, text: &str) -> bool {
        matches!(self.source_char(text), Some(' ' | '\u{00A0}'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::shaping::GlyphRun;
    use crate::span::TextRange;

    /// Builds a font-tagged run whose glyphs have the given `(cluster, advance)`
    /// pairs and byte `source` range.
    fn run(font: u32, glyphs: &[(usize, f32)], source: TextRange) -> ShapedRun {
        let shaped = glyphs
            .iter()
            .map(|&(cluster, adv)| ShapedGlyph::new(1, cluster, adv))
            .collect();
        ShapedRun::new(
            FontId::new(font),
            GlyphRun::new(shaped, TextDirection::LeftToRight, 10.0, source),
        )
    }

    #[test]
    fn flatten_preserves_run_order_and_tags_each_glyph_with_its_run() {
        let runs = vec![
            run(7, &[(0, 6.0), (1, 6.0)], TextRange::new(0, 2)),
            run(9, &[(2, 8.0)], TextRange::new(2, 3)).with_level(1),
        ];
        let stream = TaggedGlyph::flatten(&runs);
        assert_eq!(stream.len(), 3);
        assert_eq!(
            stream.iter().map(|t| t.font.get()).collect::<Vec<_>>(),
            vec![7, 7, 9]
        );
        assert_eq!(
            stream.iter().map(|t| t.level).collect::<Vec<_>>(),
            vec![0, 0, 1]
        );
        assert_eq!(
            stream.iter().map(|t| t.glyph.cluster()).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn source_char_reads_the_cluster_and_is_none_past_the_end() {
        let text = "aВ";
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 8.0), (9, 8.0)],
            TextRange::new(0, 3),
        )];
        let stream = TaggedGlyph::flatten(&runs);
        assert_eq!(stream[0].source_char(text), Some('a'));
        assert_eq!(stream[1].source_char(text), Some('В'));
        // A cluster past the end of the text has no source character.
        assert_eq!(stream[2].source_char(text), None);
    }

    #[test]
    fn only_space_and_nbsp_are_word_separators() {
        // CSS Text 3 §8.2: the separators are U+0020 and U+00A0 — a tab is not one.
        let text = " \u{00A0}\ta";
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (3, 6.0), (4, 6.0)],
            TextRange::new(0, 5),
        )];
        let stream = TaggedGlyph::flatten(&runs);
        assert!(stream[0].is_word_separator(text));
        assert!(stream[1].is_word_separator(text));
        assert!(!stream[2].is_word_separator(text));
        assert!(!stream[3].is_word_separator(text));
    }
}
