#![doc = include_str!("packed_line.doc/_module.md")]

use std::collections::VecDeque;

use crate::shaping::ShapedGlyph;

use super::break_offsets::BreakOffsets;
use super::paragraph_word::ParagraphWord;
use super::tagged_glyph::TaggedGlyph;
use super::{LineBreak, OverflowWrap, WordBreak};

#[doc = include_str!("packed_line.doc/PackedLine.md")]
pub(super) struct PackedLine {
    pub(super) glyphs: Vec<TaggedGlyph>,
    pub(super) word_count: usize,
    pub(super) mandatory_break_after: bool,
}

impl PackedLine {
    #[doc = include_str!("packed_line.doc/pack.md")]
    pub(super) fn pack(
        stream: &[TaggedGlyph],
        max_width: f32,
        breaks: &[LineBreak],
        offsets: &BreakOffsets,
        hyphen_glyph: Option<ShapedGlyph>,
        overflow_wrap: OverflowWrap,
        word_break: WordBreak,
    ) -> Vec<Self> {
        let mut queue: VecDeque<ParagraphWord> =
            ParagraphWord::split_stream(stream, breaks, offsets).into();

        // CSS Text 3 §5.2 emergency-break controls. `word-break: break-all` breaks
        // greedily at ANY grapheme-cluster boundary (a soft-wrap opportunity exists
        // between any two typographic units), so it also fills the *remaining*
        // space on the current line mid-word rather than moving the whole word
        // down. `overflow-wrap: break-word | anywhere` only breaks as a LAST RESORT
        // — when the word does not fit even alone on a full line — to keep it from
        // overflowing. `normal` + `normal` disables emergency breaking entirely, so
        // an over-wide word overflows exactly as before (byte-identical).
        let break_all = word_break.breaks_all();
        let emergency = break_all || overflow_wrap.allows_emergency_break();

        let mut lines: Vec<Self> = Vec::new();
        let mut current: Vec<TaggedGlyph> = Vec::new();
        let mut current_width = 0.0_f32;
        let mut word_count = 0_usize;
        // Whether the last word placed closed its line on a *mandatory* (forced)
        // break — used to emit a trailing empty line box after a final `\n`.
        let mut ended_on_mandatory = false;
        while let Some(word) = queue.pop_front() {
            ended_on_mandatory = false;
            // Greedy rule: a whole word that no longer fits the (non-empty) line
            // closes it. The word's own trailing collapsible whitespace **hangs**
            // (CSS Text 3 §3/§7.1), so it never counts toward the fit test —
            // otherwise a line wraps one space too early. Under `word-break:
            // break-all` we do NOT wholesale-close: the word may instead be split
            // to fill the remaining space (§5.2), handled by the emergency step.
            if !current.is_empty()
                && !break_all
                && (current_width + word.width - word.trailing_ws) > max_width
            {
                lines.push(Self {
                    glyphs: std::mem::take(&mut current),
                    word_count,
                    mandatory_break_after: false,
                });
                current_width = 0.0;
                word_count = 0;
            }

            // The word alone is wider than the space left on the (now possibly
            // fresh) line: try to break it at the last fitting opportunity.
            let remaining = max_width - current_width;
            if word.width > remaining
                && let Some(split) = word.split_at_hyphen(remaining, hyphen_glyph, offsets)
            {
                current.extend_from_slice(&split.head);
                word_count += 1;
                lines.push(Self {
                    glyphs: std::mem::take(&mut current),
                    word_count,
                    mandatory_break_after: false,
                });
                current_width = 0.0;
                word_count = 0;
                queue.push_front(split.tail);
                continue;
            }

            // Emergency (grapheme-cluster) break (CSS Text 3 §5.2). No fitting
            // hyphenation opportunity exists; if emergency breaking is enabled and
            // the word overflows the remaining space, force a grapheme break.
            if emergency && (word.width - word.trailing_ws) > remaining {
                // `break-all` breaks at any boundary; `overflow-wrap` only when the
                // word cannot fit alone on a full line (a true last resort).
                let may_break = break_all || (word.width - word.trailing_ws) > max_width;
                if may_break {
                    if let Some(split) =
                        word.emergency_split(remaining, current.is_empty(), offsets)
                    {
                        current.extend_from_slice(&split.head);
                        word_count += 1;
                        lines.push(Self {
                            glyphs: std::mem::take(&mut current),
                            word_count,
                            mandatory_break_after: false,
                        });
                        current_width = 0.0;
                        word_count = 0;
                        queue.push_front(split.tail);
                        continue;
                    }
                    if !current.is_empty() {
                        // Not even one grapheme fits the remainder: close the line
                        // and retry the word on a fresh, full-width line. This
                        // guarantees progress under `break-all`.
                        lines.push(Self {
                            glyphs: std::mem::take(&mut current),
                            word_count,
                            mandatory_break_after: false,
                        });
                        current_width = 0.0;
                        word_count = 0;
                        queue.push_front(word);
                        continue;
                    }
                    // Fresh line and no split point: the word is a single
                    // unbreakable cluster wider than the box — place it whole (it
                    // overflows by one grapheme), falling through below.
                }
            }

            current.extend_from_slice(&word.glyphs);
            current_width += word.width;
            word_count += 1;
            if word.mandatory_break_after {
                lines.push(Self {
                    glyphs: std::mem::take(&mut current),
                    word_count,
                    mandatory_break_after: true,
                });
                current_width = 0.0;
                word_count = 0;
                ended_on_mandatory = true;
            }
        }
        if !current.is_empty() {
            lines.push(Self {
                glyphs: current,
                word_count,
                mandatory_break_after: false,
            });
        } else if ended_on_mandatory {
            // A *trailing* forced break (a `\n` at end of text) opens a new, empty
            // line box (CSS Text 3 §4.1.1) — the caret needs a home on the last
            // line. `"ab\n"` is two lines: `ab`, then an empty one.
            lines.push(Self::empty());
        }
        if lines.is_empty() {
            // All content ended on a mandatory break; keep a single empty line.
            lines.push(Self::empty());
        }
        lines
    }

    #[doc = include_str!("packed_line.doc/empty.md")]
    pub(super) const fn empty() -> Self {
        Self {
            glyphs: Vec::new(),
            word_count: 0,
            mandatory_break_after: false,
        }
    }

    #[doc = include_str!("packed_line.doc/whole.md")]
    pub(super) const fn whole(glyphs: Vec<TaggedGlyph>) -> Self {
        Self {
            glyphs,
            word_count: 0,
            mandatory_break_after: false,
        }
    }

    #[doc = include_str!("packed_line.doc/width.md")]
    pub(super) fn width(&self) -> f32 {
        self.glyphs.iter().map(|t| t.glyph.x_advance()).sum()
    }

    #[doc = include_str!("packed_line.doc/retab.md")]
    pub(super) fn retab(lines: &mut [Self], text: &str, tab_width: f32) {
        for line in lines {
            let mut pen = 0.0_f32;
            for tagged in &mut line.glyphs {
                let glyph = tagged.glyph;
                if tagged.source_char(text) == Some('\t') {
                    // Advance to the next tab stop strictly past the line-local pen.
                    let stops = (pen / tab_width).floor() + 1.0;
                    let advance = (stops * tab_width - pen).max(0.0);
                    tagged.glyph = glyph.with_x_advance(advance);
                    pen += advance;
                } else {
                    pen += glyph.x_advance();
                }
            }
        }
    }

    #[doc = include_str!("packed_line.doc/hang_trailing.md")]
    pub(super) fn hang_trailing(
        &mut self,
        text: &str,
        offsets: &BreakOffsets,
        letter_spacing: f32,
        word_spacing: f32,
    ) {
        let glyphs = &mut self.glyphs;
        let last = zero_trailing_hang_run(glyphs, offsets);
        trim_trailing_tracking(glyphs, last, text, letter_spacing, word_spacing);
    }
}

fn zero_trailing_hang_run(glyphs: &mut [TaggedGlyph], offsets: &BreakOffsets) -> usize {
    let mut last = glyphs.len();
    while last > 0 && offsets.hangs(glyphs[last - 1].glyph.cluster()) {
        let glyph = glyphs[last - 1].glyph;
        glyphs[last - 1].glyph = glyph.with_x_advance(0.0);
        last -= 1;
    }
    last
}

fn trim_trailing_tracking(
    glyphs: &mut [TaggedGlyph],
    last: usize,
    text: &str,
    letter_spacing: f32,
    word_spacing: f32,
) {
    if (letter_spacing != 0.0 || word_spacing != 0.0) && last > 0 {
        let tagged = glyphs[last - 1];
        let mut advance = tagged.glyph.x_advance() - letter_spacing;
        if tagged.is_word_separator(text) {
            advance -= word_spacing;
        }
        glyphs[last - 1].glyph = tagged.glyph.with_x_advance(advance);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::font::FontId;
    use crate::layout::{NullHyphenator, WhiteSpace};
    use crate::shaping::{GlyphRun, ShapedRun, TextDirection};
    use crate::span::TextRange;

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
    fn a_tab_on_a_wrapped_line_measures_from_that_lines_own_left_edge() {
        // "a\tb": the tab's advance was measured from the paragraph origin (pen
        // 6 → stop 8, advance 2). On a line that starts at the tab, the pen is 0,
        // so the tab must advance a full stop instead.
        let text = "a\tb";
        let mut lines = vec![PackedLine {
            glyphs: stream(&[(1, 2.0), (2, 6.0)], TextRange::new(1, 3)),
            word_count: 1,
            mandatory_break_after: false,
        }];
        PackedLine::retab(&mut lines, text, 8.0);
        assert_eq!(lines[0].glyphs[0].glyph.x_advance(), 8.0);
        assert_eq!(lines[0].glyphs[1].glyph.x_advance(), 6.0);
    }

    #[test]
    fn a_trailing_collapsible_space_is_zeroed_but_keeps_its_cluster() {
        let text = "ab ";
        let offsets = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Normal);
        let mut line = PackedLine {
            glyphs: stream(&[(0, 6.0), (1, 6.0), (2, 4.0)], TextRange::new(0, 3)),
            word_count: 1,
            mandatory_break_after: false,
        };
        line.hang_trailing(text, &offsets, 0.0, 0.0);
        assert_eq!(line.width(), 12.0);
        // The glyph itself survives, so hit-testing still sees the space.
        assert_eq!(line.glyphs.len(), 3);
        assert_eq!(line.glyphs[2].glyph.cluster(), 2);
    }

    #[test]
    fn trailing_letter_spacing_is_trimmed_from_the_last_remaining_cluster() {
        // Both glyphs carry a folded-in 2.0 of letter-spacing; only the last
        // one's hangs off the line's end.
        let text = "ab";
        let offsets = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Normal);
        let mut line = PackedLine {
            glyphs: stream(&[(0, 8.0), (1, 8.0)], TextRange::new(0, 2)),
            word_count: 1,
            mandatory_break_after: false,
        };
        line.hang_trailing(text, &offsets, 2.0, 0.0);
        assert_eq!(line.width(), 14.0);
    }

    #[test]
    fn an_empty_stream_still_yields_one_empty_line() {
        let text = "";
        let offsets = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Normal);
        let lines = PackedLine::pack(
            &[],
            100.0,
            &[],
            &offsets,
            None,
            OverflowWrap::Normal,
            WordBreak::Normal,
        );
        assert_eq!(lines.len(), 1);
        assert!(lines[0].glyphs.is_empty());
    }
}
