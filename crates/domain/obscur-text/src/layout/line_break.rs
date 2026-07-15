//! Line-break opportunities discovered in a run of text (spec §5.10).

use super::WhiteSpace;

/// The kind of a [`LineBreak`] opportunity (spec §5.10).
///
/// A [`Soft`](Self::Soft) break is a *permitted* wrap point (for example after a
/// run of whitespace) that a line-breaking pass may or may not take; a
/// [`Hard`](Self::Hard) break is *mandatory* (for example a `'\n'`) and always
/// ends the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BreakKind {
    /// A permitted (optional) wrap point, e.g. after a space.
    Soft,
    /// A mandatory break, e.g. a line feed.
    Hard,
}

/// A single line-break opportunity: a byte offset in the source text where a
/// line may (soft) or must (hard) break (spec §5.10).
///
/// The [`offset`](Self::offset) is a **byte** index into the UTF-8 source and
/// marks the position at which the following text would start a new line. Breaks
/// order by `offset` first, then by [`kind`](Self::kind) (soft before hard),
/// which the derived [`Ord`] provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineBreak {
    /// The byte offset in the source where the break may or must occur.
    offset: usize,
    /// Whether the break is optional ([`BreakKind::Soft`]) or mandatory
    /// ([`BreakKind::Hard`]).
    kind: BreakKind,
}

impl LineBreak {
    /// Creates a [`Soft`](BreakKind::Soft) (permitted) break at byte `offset`.
    #[must_use]
    pub const fn soft(offset: usize) -> Self {
        Self {
            offset,
            kind: BreakKind::Soft,
        }
    }

    /// Creates a [`Hard`](BreakKind::Hard) (mandatory) break at byte `offset`.
    #[must_use]
    pub const fn hard(offset: usize) -> Self {
        Self {
            offset,
            kind: BreakKind::Hard,
        }
    }

    /// Returns the byte offset in the source where the break occurs.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the break's [`BreakKind`].
    #[must_use]
    pub const fn kind(&self) -> BreakKind {
        self.kind
    }

    /// Returns whether this break is mandatory ([`BreakKind::Hard`]).
    #[must_use]
    pub const fn is_mandatory(&self) -> bool {
        matches!(self.kind, BreakKind::Hard)
    }

    /// Finds the line-break opportunities in `text` under a `white_space` mode
    /// (CSS Text 3 §3 white-space table, §5.10).
    ///
    /// This is a deliberately **simplified** approximation of the Unicode line
    /// breaking algorithm (UAX #14); full, tailored breaking is an adapter
    /// concern. The `white_space` mode selects which breaks a run yields:
    ///
    /// - a [`Hard`](BreakKind::Hard) break just after each `'\n'` (its offset is
    ///   the byte index of the character following the newline) — but **only**
    ///   when [`white_space.preserves_segment_breaks()`](WhiteSpace::preserves_segment_breaks)
    ///   (`pre` / `pre-wrap` / `pre-line`). Under `normal` / `nowrap` a segment
    ///   break has already been collapsed to a space at the pre-shape seam, so it
    ///   is treated here as ordinary whitespace;
    /// - a [`Soft`](BreakKind::Soft) break after each maximal run of whitespace,
    ///   at the byte offset of the first non-whitespace character that follows it
    ///   (the break sits *before* the next word) — but **only** when
    ///   [`white_space.wraps()`](WhiteSpace::wraps) (`normal` / `pre-wrap` /
    ///   `pre-line`). Under `nowrap` / `pre` no soft break is emitted, so the
    ///   wrapper lays each segment on a single (possibly overflowing) line.
    ///
    /// A whitespace run that contains a preserved segment break yields only the
    /// mandatory break, never a duplicate soft break at the same position.
    /// Offsets are byte-correct for UTF-8 because iteration is by
    /// [`str::char_indices`]. The returned breaks are in ascending offset order.
    #[must_use]
    pub fn opportunities(text: &str, white_space: WhiteSpace) -> Vec<Self> {
        let mut breaks = Vec::new();
        let wraps = white_space.wraps();
        let preserves_breaks = white_space.preserves_segment_breaks();
        // Whether the previous character was whitespace, and whether that
        // whitespace run contained a preserved (hard) break, which already
        // emitted a mandatory break and suppresses the run's soft break.
        let mut in_whitespace = false;
        let mut whitespace_had_hard_break = false;
        for (index, ch) in text.char_indices() {
            if ch == '\n' && preserves_breaks {
                breaks.push(Self::hard(index + ch.len_utf8()));
                whitespace_had_hard_break = true;
            }
            if ch.is_whitespace() {
                in_whitespace = true;
            } else {
                // Transition from a whitespace run into a word: a soft break
                // sits before this word, unless the mode disallows soft wrapping
                // or a hard break already covers it.
                if in_whitespace && wraps && !whitespace_had_hard_break {
                    breaks.push(Self::soft(index));
                }
                in_whitespace = false;
                whitespace_had_hard_break = false;
            }
        }
        breaks
    }
}

/// Whether `ch` is a CJK **ideograph** — a large, well-defined set of Unicode
/// blocks whose characters may line-break between one another by default (UAX #14
/// class `ID`; CSS Text 3 §5.2 `word-break: normal`).
///
/// This is the Han-ideograph subset of UAX#14 class `ID`: the CJK Unified
/// Ideographs and their extensions plus the compatibility ideographs. Kana and
/// the CJK symbol/punctuation blocks (which carry additional tailoring rules) are
/// deliberately excluded so the default breaker stays simple and deterministic;
/// full UAX#14 tailoring is an adapter concern (see [`LineBreaker`]).
#[must_use]
pub(crate) fn is_cjk_ideograph(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF      // CJK Unified Ideographs Extension A
        | 0x4E00..=0x9FFF    // CJK Unified Ideographs
        | 0xF900..=0xFAFF    // CJK Compatibility Ideographs
        | 0x2_0000..=0x2_A6DF // Extension B
        | 0x2_A700..=0x2_EBEF // Extensions C–F
        | 0x2_F800..=0x2_FA1F // CJK Compatibility Ideographs Supplement
        | 0x3_0000..=0x3_134F // Extension G
    )
}

/// A replaceable **line-break-opportunity port** (spec §5.10, UAX #14).
///
/// This is the seam through which a full Unicode line-breaking (UAX #14) or CJK
/// adapter can supply break opportunities without changing the wrapping engine —
/// the exact analogue of the [`Hyphenator`](super::Hyphenator) port for interior
/// hyphenation. The domain wraps text through a `&dyn LineBreaker`; the default
/// implementation ([`DefaultLineBreaker`]) reproduces the historical
/// whitespace-and-forced-break behaviour plus basic inter-ideograph CJK breaks,
/// while a host may install a tailored adapter behind the xtask boundary.
///
/// # Contract
///
/// - The returned breaks are byte offsets into `text` (UTF-8), each marking a
///   position at which the following text may (soft) or must (hard) start a new
///   line, in **ascending** offset order.
/// - The result must be **deterministic**: the same `text` and `white_space`
///   always yield the same opportunities, so the measure and compose passes agree
///   on the same lines.
///
/// It is deliberately **object-safe** so the runtime can hold a
/// `Box<dyn LineBreaker>` / pass a `&dyn LineBreaker`.
pub trait LineBreaker {
    /// Returns the line-break opportunities in `text` under a `white_space` mode,
    /// in ascending offset order (spec §5.10, UAX #14).
    fn opportunities(&self, text: &str, white_space: WhiteSpace) -> Vec<LineBreak>;
}

/// The domain-default [`LineBreaker`]: the simplified whitespace / forced-break
/// opportunities of [`LineBreak::opportunities`], **plus** a soft break between
/// two adjacent CJK ideographs so unspaced CJK text wraps out of the box (spec
/// §5.10, UAX #14 class `ID`).
///
/// For text containing **no** CJK ideographs — the common Latin / Cyrillic /
/// whitespace-delimited case — this is *byte-identical* to
/// [`LineBreak::opportunities`]: the CJK pass adds nothing and the untouched
/// vector is returned. A host installs a real [`LineBreaker`] adapter for full
/// UAX#14 tailoring (kana, punctuation, `word-break: keep-all` letter clustering,
/// …); `keep-all`'s suppression of these inter-ideograph breaks is applied by the
/// wrapper, so it is honoured even with this default breaker.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DefaultLineBreaker;

impl LineBreaker for DefaultLineBreaker {
    fn opportunities(&self, text: &str, white_space: WhiteSpace) -> Vec<LineBreak> {
        let mut breaks = LineBreak::opportunities(text, white_space);
        // A break opportunity sits between two adjacent CJK ideographs, but only
        // where soft wrapping is permitted (§3). This is additive: for text with
        // no ideograph pair the loop pushes nothing and `breaks` is returned
        // untouched (byte-identical to `LineBreak::opportunities`).
        if white_space.wraps() {
            let before = breaks.len();
            let mut prev: Option<char> = None;
            for (index, ch) in text.char_indices() {
                if let Some(prev_ch) = prev
                    && is_cjk_ideograph(prev_ch)
                    && is_cjk_ideograph(ch)
                {
                    breaks.push(LineBreak::soft(index));
                }
                prev = Some(ch);
            }
            if breaks.len() != before {
                // Merge the appended CJK breaks back into ascending order and drop
                // any that coincide with an existing opportunity.
                breaks.sort_unstable();
                breaks.dedup();
            }
        }
        breaks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_and_accessors() {
        let soft = LineBreak::soft(4);
        assert_eq!(soft.offset(), 4);
        assert_eq!(soft.kind(), BreakKind::Soft);
        assert!(!soft.is_mandatory());

        let hard = LineBreak::hard(7);
        assert_eq!(hard.offset(), 7);
        assert_eq!(hard.kind(), BreakKind::Hard);
        assert!(hard.is_mandatory());
    }

    #[test]
    fn ordering_is_offset_then_kind() {
        assert!(LineBreak::soft(2) < LineBreak::hard(4));
        // Same offset: Soft sorts before Hard.
        assert!(LineBreak::soft(4) < LineBreak::hard(4));
    }

    #[test]
    fn opportunities_of_a_b_newline_c_under_preserving_breaks() {
        // "a b\nc" with a mode that preserves segment breaks (`pre-line`): soft
        // break before "b" (byte 2), hard break after "\n" (byte 4).
        let breaks = LineBreak::opportunities("a b\nc", WhiteSpace::PreLine);
        assert_eq!(breaks, vec![LineBreak::soft(2), LineBreak::hard(4)]);
    }

    #[test]
    fn newline_run_yields_only_hard_break_when_preserved() {
        // The word after a preserved newline gets no duplicate soft break.
        let breaks = LineBreak::opportunities("a\nb", WhiteSpace::PreLine);
        assert_eq!(breaks, vec![LineBreak::hard(2)]);
    }

    #[test]
    fn soft_break_after_run_of_spaces_is_utf8_byte_correct() {
        // "é  x": 'é' is two bytes (0..2), two spaces (2, 3), 'x' at byte 4.
        let breaks = LineBreak::opportunities("é  x", WhiteSpace::Normal);
        assert_eq!(breaks, vec![LineBreak::soft(4)]);
    }

    #[test]
    fn no_opportunities_in_a_single_word() {
        assert!(LineBreak::opportunities("hello", WhiteSpace::Normal).is_empty());
    }

    #[test]
    fn normal_treats_newline_as_soft_whitespace_not_hard() {
        // Under `normal` a `\n` is NOT preserved (§3): it is ordinary whitespace,
        // so "a\nb" yields a single soft break before "b" (byte 2), no hard break.
        let breaks = LineBreak::opportunities("a\nb", WhiteSpace::Normal);
        assert_eq!(breaks, vec![LineBreak::soft(2)]);
    }

    #[test]
    fn nowrap_yields_no_breaks_at_all() {
        // `nowrap`: no soft wrap and (segment breaks collapsed) no hard break —
        // the whole run is one line.
        assert!(LineBreak::opportunities("a b\nc", WhiteSpace::Nowrap).is_empty());
    }

    #[test]
    fn pre_yields_only_hard_breaks_no_soft() {
        // `pre`: preserve segment breaks (hard at byte 4) but no soft wrap, so the
        // space before "b" yields no soft break.
        let breaks = LineBreak::opportunities("a b\nc", WhiteSpace::Pre);
        assert_eq!(breaks, vec![LineBreak::hard(4)]);
    }

    #[test]
    fn pre_wrap_yields_both_soft_and_hard() {
        // `pre-wrap`: soft before "b" (byte 2) and hard after "\n" (byte 4).
        let breaks = LineBreak::opportunities("a b\nc", WhiteSpace::PreWrap);
        assert_eq!(breaks, vec![LineBreak::soft(2), LineBreak::hard(4)]);
    }

    #[test]
    fn default_breaker_reproduces_opportunities_for_non_cjk_text() {
        // Requirement (d): the port default is byte-identical to
        // `LineBreak::opportunities` for every whitespace mode on Latin text.
        for text in ["hello", "a b\nc", "aa bb cc", "é  x", "ab"] {
            for mode in [
                WhiteSpace::Normal,
                WhiteSpace::Nowrap,
                WhiteSpace::Pre,
                WhiteSpace::PreWrap,
                WhiteSpace::PreLine,
            ] {
                assert_eq!(
                    DefaultLineBreaker.opportunities(text, mode),
                    LineBreak::opportunities(text, mode),
                    "{text:?} / {mode:?}"
                );
            }
        }
    }

    #[test]
    fn default_breaker_adds_soft_break_between_adjacent_cjk_ideographs() {
        // "中文字" — three CJK ideographs (3 bytes each): soft breaks before the
        // 2nd (byte 3) and 3rd (byte 6) so unspaced CJK wraps out of the box.
        let breaks = DefaultLineBreaker.opportunities("中文字", WhiteSpace::Normal);
        assert_eq!(breaks, vec![LineBreak::soft(3), LineBreak::soft(6)]);
        // A non-wrapping mode (`nowrap`) yields no CJK soft breaks.
        assert!(
            DefaultLineBreaker
                .opportunities("中文字", WhiteSpace::Nowrap)
                .is_empty()
        );
    }

    #[test]
    fn default_breaker_is_object_safe() {
        let breaker: &dyn LineBreaker = &DefaultLineBreaker;
        assert!(
            breaker
                .opportunities("hello", WhiteSpace::Normal)
                .is_empty()
        );
    }

    #[test]
    fn cjk_ideograph_range() {
        assert!(is_cjk_ideograph('中'));
        assert!(is_cjk_ideograph('字'));
        assert!(!is_cjk_ideograph('a'));
        assert!(!is_cjk_ideograph('Я'));
        // Hiragana is deliberately excluded from the simplified default set.
        assert!(!is_cjk_ideograph('あ'));
    }
}
