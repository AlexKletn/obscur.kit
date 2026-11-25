//! A paragraph's unbreakable unit: the tagged glyphs between two adjacent break
//! opportunities, and the interior points at which it may still be split
//! (spec §5.10).

use crate::shaping::ShapedGlyph;

use super::LineBreak;
use super::break_offsets::BreakOffsets;
use super::hyphen_point::HyphenPoint;
use super::tagged_glyph::TaggedGlyph;
use super::word_split::WordSplit;

/// A word: the tagged glyphs between two adjacent break opportunities, its
/// advance width, whether a mandatory break follows it, and the interior
/// hyphenation points at which it may be split. A word may contain glyphs from
/// more than one font.
pub(super) struct ParagraphWord {
    pub(super) glyphs: Vec<TaggedGlyph>,
    pub(super) width: f32,
    #[doc = include_str!("../../docs/api/layout/paragraph_word.doc/trailing_ws.md")]
    pub(super) trailing_ws: f32,
    pub(super) mandatory_break_after: bool,
    /// The in-word split points (from the hyphenator or author soft hyphens),
    /// ascending by [`head_width`](HyphenPoint::head_width).
    hyphens: Vec<HyphenPoint>,
}

impl ParagraphWord {
    #[doc = include_str!("../../docs/api/layout/paragraph_word.doc/split_stream.md")]
    pub(super) fn split_stream(
        stream: &[TaggedGlyph],
        breaks: &[LineBreak],
        offsets: &BreakOffsets,
    ) -> Vec<Self> {
        let mut words: Vec<Self> = Vec::new();
        let mut glyphs: Vec<TaggedGlyph> = Vec::new();
        // Breaks are ascending by offset.
        let mut next_break = 0usize;
        for tagged in stream {
            let cluster = tagged.glyph.cluster();
            // Consume every break at or before this glyph; the strongest (hard)
            // kind wins for the word we are about to close.
            let mut close = false;
            let mut close_hard = false;
            while next_break < breaks.len() && breaks[next_break].offset() <= cluster {
                close = true;
                close_hard |= breaks[next_break].is_mandatory();
                next_break += 1;
            }
            if close {
                if !glyphs.is_empty() {
                    words.push(Self::new(std::mem::take(&mut glyphs), close_hard, offsets));
                } else if close_hard {
                    // A mandatory break with nothing before it (leading or repeated
                    // `\n`): emit an empty forced-break word → an empty line box.
                    words.push(Self::new(Vec::new(), true, offsets));
                }
            }
            glyphs.push(*tagged);
        }
        // Breaks after the last glyph (offset > every cluster): a trailing `\n`
        // whose cluster no glyph carries still forces a break on the final word.
        let mut trailing_hard = false;
        while next_break < breaks.len() {
            trailing_hard |= breaks[next_break].is_mandatory();
            next_break += 1;
        }
        if !glyphs.is_empty() {
            words.push(Self::new(glyphs, trailing_hard, offsets));
        } else if trailing_hard {
            words.push(Self::new(Vec::new(), true, offsets));
        }
        words
    }

    #[doc = include_str!("../../docs/api/layout/paragraph_word.doc/new.md")]
    pub(super) fn new(
        glyphs: Vec<TaggedGlyph>,
        mandatory_break_after: bool,
        offsets: &BreakOffsets,
    ) -> Self {
        let width: f32 = glyphs.iter().map(|t| t.glyph.x_advance()).sum();
        // The trailing run of collapsible-whitespace advance that hangs when this
        // word ends a line (CSS Text 3 §3/§7.1).
        let trailing_ws: f32 = glyphs
            .iter()
            .rev()
            .take_while(|t| offsets.hangs(t.glyph.cluster()))
            .map(|t| t.glyph.x_advance())
            .sum();
        let mut hyphens: Vec<HyphenPoint> = Vec::new();
        let mut head_width = 0.0_f32;
        for (index, tagged) in glyphs.iter().enumerate() {
            let cluster = tagged.glyph.cluster();
            // An algorithmic break resumes the tail *at* this glyph.
            if index >= 1 && offsets.is_hyphen_point(cluster) {
                hyphens.push(HyphenPoint {
                    head_end: index,
                    tail_start: index,
                    head_width,
                    break_cluster: cluster,
                });
            }
            // A soft hyphen breaks *after* this (invisible) glyph, which is
            // dropped; require a non-empty head and a non-empty tail.
            if offsets.is_soft_hyphen(cluster) && index >= 1 && index + 1 < glyphs.len() {
                hyphens.push(HyphenPoint {
                    head_end: index,
                    tail_start: index + 1,
                    head_width,
                    break_cluster: cluster,
                });
            }
            head_width += tagged.glyph.x_advance();
        }
        hyphens.sort_by(|a, b| {
            a.head_width
                .partial_cmp(&b.head_width)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.head_end.cmp(&b.head_end))
        });
        Self {
            glyphs,
            width,
            trailing_ws,
            mandatory_break_after,
            hyphens,
        }
    }

    #[doc = include_str!("../../docs/api/layout/paragraph_word.doc/split_at_hyphen.md")]
    pub(super) fn split_at_hyphen(
        &self,
        limit: f32,
        hyphen_glyph: Option<ShapedGlyph>,
        offsets: &BreakOffsets,
    ) -> Option<WordSplit> {
        let hyphen_adv = hyphen_glyph.map_or(0.0, |g| g.x_advance());
        // Points are ascending by head width; take the widest that still fits.
        let point = self
            .hyphens
            .iter()
            .rev()
            .find(|p| p.head_width + hyphen_adv <= limit)
            .copied()?;

        let mut head: Vec<TaggedGlyph> = self.glyphs[..point.head_end].to_vec();
        if let (Some(glyph), Some(last)) = (hyphen_glyph, head.last().copied()) {
            // The hyphen inherits the trailing head glyph's font/direction/level
            // so it segments and bidi-reorders with the head, and carries the
            // break's source cluster for hit-testing.
            let hyphen = ShapedGlyph::new(glyph.glyph_id(), point.break_cluster, glyph.x_advance())
                .with_offset(glyph.x_offset(), glyph.y_offset());
            head.push(TaggedGlyph {
                glyph: hyphen,
                ..last
            });
        }

        let tail = Self::new(
            self.glyphs[point.tail_start..].to_vec(),
            self.mandatory_break_after,
            offsets,
        );
        Some(WordSplit { head, tail })
    }

    #[doc = include_str!("../../docs/api/layout/paragraph_word.doc/emergency_split.md")]
    pub(super) fn emergency_split(
        &self,
        limit: f32,
        line_empty: bool,
        offsets: &BreakOffsets,
    ) -> Option<WordSplit> {
        let glyphs = &self.glyphs;
        if glyphs.len() < 2 {
            return None;
        }
        let mut widest_fitting_end = 0usize;
        let mut first_cluster_end = 0usize;
        let mut head_width = 0.0_f32;
        let mut index = 0usize;
        while index < glyphs.len() {
            let cluster = glyphs[index].glyph.cluster();
            let mut width = 0.0_f32;
            while index < glyphs.len() && glyphs[index].glyph.cluster() == cluster {
                width += glyphs[index].glyph.x_advance();
                index += 1;
            }
            if first_cluster_end == 0 {
                first_cluster_end = index;
            }
            if head_width + width <= limit {
                head_width += width;
                widest_fitting_end = index;
            } else {
                break;
            }
        }
        let head_end = if widest_fitting_end == 0 {
            if line_empty {
                first_cluster_end
            } else {
                return None;
            }
        } else {
            widest_fitting_end
        };
        if head_end == 0 || head_end >= glyphs.len() {
            return None;
        }
        let head = glyphs[..head_end].to_vec();
        let tail = Self::new(
            glyphs[head_end..].to_vec(),
            self.mandatory_break_after,
            offsets,
        );
        Some(WordSplit { head, tail })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::font::FontId;
    use crate::layout::{Hyphenator, NullHyphenator, WhiteSpace};
    use crate::shaping::{GlyphRun, ShapedRun, TextDirection};
    use crate::span::TextRange;

    /// A hyphenator that proposes every offset in `offsets` inside every word.
    struct StubHyphenator {
        offsets: Vec<usize>,
    }

    impl Hyphenator for StubHyphenator {
        fn hyphenate(&self, word: &str) -> Vec<usize> {
            self.offsets
                .iter()
                .copied()
                .filter(|&offset| offset < word.len())
                .collect()
        }
    }

    /// Flattens one LTR run of `(cluster, advance)` pairs into a tagged stream.
    fn stream(glyphs: &[(usize, f32)], source: TextRange) -> Vec<TaggedGlyph> {
        let shaped = glyphs
            .iter()
            .map(|&(cluster, adv)| ShapedGlyph::new(1, cluster, adv))
            .collect();
        let run = ShapedRun::new(
            FontId::new(0),
            GlyphRun::new(shaped, TextDirection::LeftToRight, 10.0, source),
        );
        TaggedGlyph::flatten(&[run])
    }

    #[test]
    fn a_soft_break_closes_the_word_before_the_glyph_it_sits_at() {
        // "ab cd": the soft break at byte 3 closes "ab " before the 'c' glyph.
        let text = "ab cd";
        let offsets = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Normal);
        let glyphs = stream(
            &[(0, 6.0), (1, 6.0), (2, 4.0), (3, 6.0), (4, 6.0)],
            TextRange::new(0, 5),
        );
        let words = ParagraphWord::split_stream(&glyphs, &[LineBreak::soft(3)], &offsets);
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].glyphs.len(), 3);
        assert_eq!(words[1].glyphs.len(), 2);
        // The first word owns the space that follows it, and that space hangs.
        assert_eq!(words[0].width, 16.0);
        assert_eq!(words[0].trailing_ws, 4.0);
        assert_eq!(words[1].trailing_ws, 0.0);
    }

    #[test]
    fn a_trailing_hard_break_past_every_glyph_still_marks_the_last_word() {
        // "ab\n" under `pre`: the hard break sits at byte 3, which no glyph
        // carries, and must still force a break after "ab".
        let text = "ab\n";
        let offsets = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Pre);
        let glyphs = stream(&[(0, 6.0), (1, 6.0)], TextRange::new(0, 2));
        let words = ParagraphWord::split_stream(&glyphs, &[LineBreak::hard(3)], &offsets);
        assert_eq!(words.len(), 1);
        assert!(words[0].mandatory_break_after);
    }

    #[test]
    fn the_widest_fitting_hyphen_point_is_taken_and_carries_the_break_cluster() {
        // "abcd" with interior points before 'b', 'c' and 'd'; a limit of 13
        // admits the head "ab" plus the hyphen, not the wider "abc".
        let text = "abcd";
        let offsets = BreakOffsets::collect(
            text,
            &[],
            &StubHyphenator {
                offsets: vec![1, 2, 3],
            },
            WhiteSpace::Normal,
        );
        let glyphs = stream(
            &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0)],
            TextRange::new(0, 4),
        );
        let word = ParagraphWord::new(glyphs, false, &offsets);
        let hyphen = ShapedGlyph::new(99, 0, 1.0);
        let split = word
            .split_at_hyphen(13.0, Some(hyphen), &offsets)
            .expect("a fitting interior point");
        // Head "ab" + the hyphen glyph, which carries the break's source cluster.
        assert_eq!(split.head.len(), 3);
        assert_eq!(split.head[2].glyph.glyph_id(), 99);
        assert_eq!(split.head[2].glyph.cluster(), 2);
        assert_eq!(split.tail.glyphs.len(), 2);
    }

    #[test]
    fn no_fitting_hyphen_point_leaves_the_word_whole() {
        let text = "abcd";
        let offsets = BreakOffsets::collect(
            text,
            &[],
            &StubHyphenator { offsets: vec![2] },
            WhiteSpace::Normal,
        );
        let glyphs = stream(
            &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0)],
            TextRange::new(0, 4),
        );
        let word = ParagraphWord::new(glyphs, false, &offsets);
        assert!(word.split_at_hyphen(5.0, None, &offsets).is_none());
    }

    #[test]
    fn an_emergency_split_never_cuts_inside_a_multi_glyph_cluster() {
        // Three glyphs, two clusters: 0 (two glyphs, a base plus a mark) and 1.
        // A limit that would fit "one and a half glyphs" still cuts at the
        // cluster boundary, so the head is the whole first cluster.
        let text = "ab";
        let offsets = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Normal);
        let glyphs = stream(&[(0, 6.0), (0, 6.0), (1, 6.0)], TextRange::new(0, 2));
        let word = ParagraphWord::new(glyphs, false, &offsets);
        let split = word
            .emergency_split(15.0, true, &offsets)
            .expect("a cluster boundary head");
        assert_eq!(split.head.len(), 2);
        assert_eq!(split.tail.glyphs.len(), 1);
    }

    #[test]
    fn an_emergency_split_on_a_partial_line_declines_rather_than_overflowing() {
        // Not even the first cluster fits, and the line is not empty: decline so
        // the caller closes the line and retries at full width.
        let text = "ab";
        let offsets = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Normal);
        let glyphs = stream(&[(0, 6.0), (1, 6.0)], TextRange::new(0, 2));
        let word = ParagraphWord::new(glyphs, false, &offsets);
        assert!(word.emergency_split(2.0, false, &offsets).is_none());
        // On a fresh line the first cluster is forced on, overflowing by one
        // grapheme rather than looping forever.
        let split = word
            .emergency_split(2.0, true, &offsets)
            .expect("forced progress");
        assert_eq!(split.head.len(), 1);
    }
}
