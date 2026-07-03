//! Line-break opportunities discovered in a run of text (spec §5.10).

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

    /// Finds the line-break opportunities in `text` (spec §5.10).
    ///
    /// This is a deliberately **simplified** approximation of the Unicode line
    /// breaking algorithm (UAX #14); full, tailored breaking is an adapter
    /// concern. The rules are:
    ///
    /// - a [`Hard`](BreakKind::Hard) break just after each `'\n'` (its offset is
    ///   the byte index of the character following the newline), and
    /// - a [`Soft`](BreakKind::Soft) break after each maximal run of whitespace,
    ///   at the byte offset of the first non-whitespace character that follows it
    ///   (i.e. the break sits *before* the next word).
    ///
    /// A whitespace run that contains a newline yields only the mandatory break,
    /// never a duplicate soft break at the same position. Offsets are byte-correct
    /// for UTF-8 because iteration is by [`str::char_indices`]. The returned
    /// breaks are in ascending offset order.
    #[must_use]
    pub fn opportunities(text: &str) -> Vec<Self> {
        let mut breaks = Vec::new();
        // Whether the previous character was whitespace, and whether that
        // whitespace run contained a newline (which already emits a hard break).
        let mut in_whitespace = false;
        let mut whitespace_had_newline = false;
        for (index, ch) in text.char_indices() {
            if ch == '\n' {
                breaks.push(Self::hard(index + ch.len_utf8()));
            }
            if ch.is_whitespace() {
                in_whitespace = true;
                if ch == '\n' {
                    whitespace_had_newline = true;
                }
            } else {
                // Transition from a whitespace run into a word: a soft break
                // sits before this word, unless a hard break already covers it.
                if in_whitespace && !whitespace_had_newline {
                    breaks.push(Self::soft(index));
                }
                in_whitespace = false;
                whitespace_had_newline = false;
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
    fn opportunities_of_a_b_newline_c() {
        // "a b\nc": soft break before "b" (byte 2), hard break after "\n" (byte 4).
        let breaks = LineBreak::opportunities("a b\nc");
        assert_eq!(breaks, vec![LineBreak::soft(2), LineBreak::hard(4)]);
    }

    #[test]
    fn newline_run_yields_only_hard_break() {
        // The word after a newline gets no duplicate soft break.
        let breaks = LineBreak::opportunities("a\nb");
        assert_eq!(breaks, vec![LineBreak::hard(2)]);
    }

    #[test]
    fn soft_break_after_run_of_spaces_is_utf8_byte_correct() {
        // "é  x": 'é' is two bytes (0..2), two spaces (2, 3), 'x' at byte 4.
        let breaks = LineBreak::opportunities("é  x");
        assert_eq!(breaks, vec![LineBreak::soft(4)]);
    }

    #[test]
    fn no_opportunities_in_a_single_word() {
        assert!(LineBreak::opportunities("hello").is_empty());
    }
}
