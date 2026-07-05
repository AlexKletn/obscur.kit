//! The [`UnicodeTextItemizer`] adapter implementing [`TextItemizer`] over real
//! Unicode script and bidi tables (spec §5.10).

use obscur_text::shaping::{ScriptTag, TextDirection, TextItem, TextItemizer};
use obscur_text::span::TextRange;
use unicode_bidi::{BidiInfo, Level};

use crate::script_mapping::script_tag_for;

/// A real [`TextItemizer`] that splits text by bidi level and script (spec §5.10).
///
/// Itemization runs in two nested passes over the source text in logical order:
///
/// 1. **Bidi.** `unicode-bidi` resolves the Unicode Bidirectional Algorithm for
///    the whole string, seeded by the caller's base direction, producing one
///    embedding [`Level`] per byte. A run is cut wherever the level changes; the
///    run's [`direction`](TextItem::direction) follows the level parity (even =
///    left-to-right, odd = right-to-left) and its [`level`](TextItem::level) is
///    that embedding level.
/// 2. **Script.** Within a single-level run the text is cut again wherever the
///    `unicode-script` script changes. Script-neutral characters (`Common` /
///    `Inherited` — spaces, punctuation, digits, combining marks) never start a
///    new run: they extend the current one, and a leading neutral prefix adopts
///    the script of the run that follows it.
///
/// The emitted items are contiguous, non-overlapping, and cover `0..text.len()`
/// with no gaps, so every range lands on a UTF-8 character boundary. An empty
/// string yields a single empty item `0..0` carrying the base direction (level
/// `0` for LTR, `1` for RTL) and script [`ScriptTag::COMMON`], matching the
/// reference `SingleRunItemizer`.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnicodeTextItemizer;

impl UnicodeTextItemizer {
    /// Creates a `UnicodeTextItemizer`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// The run currently being accumulated during a single left-to-right pass.
struct PendingRun {
    /// Byte offset where the run starts.
    start: usize,
    /// The run's uniform bidi embedding level.
    level: u8,
    /// The run's resolved script, or `None` while only neutral characters have
    /// been seen (a leading neutral prefix awaiting a concrete script).
    script: Option<ScriptTag>,
}

impl PendingRun {
    /// Finalizes the run as a [`TextItem`] spanning `self.start..end`.
    fn finish(&self, end: usize) -> TextItem {
        let direction = if self.level.is_multiple_of(2) {
            TextDirection::LeftToRight
        } else {
            TextDirection::RightToLeft
        };
        TextItem::new(
            TextRange::new(self.start, end),
            self.script.unwrap_or(ScriptTag::COMMON),
            direction,
            self.level,
        )
    }
}

impl TextItemizer for UnicodeTextItemizer {
    fn itemize(&self, text: &str, base_direction: TextDirection) -> Vec<TextItem> {
        if text.is_empty() {
            let level = u8::from(base_direction.is_rtl());
            return vec![TextItem::new(
                TextRange::empty(0),
                ScriptTag::COMMON,
                base_direction,
                level,
            )];
        }

        // One embedding level per byte, seeded by the caller's base direction.
        let base_level = if base_direction.is_rtl() {
            Level::rtl()
        } else {
            Level::ltr()
        };
        let bidi = BidiInfo::new(text, Some(base_level));
        let levels = &bidi.levels;

        let mut items = Vec::new();
        let mut run: Option<PendingRun> = None;

        for (offset, ch) in text.char_indices() {
            let level = levels[offset].number();
            let char_script = script_tag_for(ch);

            match run.as_mut() {
                None => {
                    run = Some(PendingRun {
                        start: offset,
                        level,
                        script: char_script,
                    });
                }
                Some(current) => {
                    let level_changed = level != current.level;
                    let script_changed = match (current.script, char_script) {
                        (Some(a), Some(b)) => a != b,
                        // A neutral character never forces a break, and a
                        // concrete character extends a still-undetermined run.
                        _ => false,
                    };

                    if level_changed || script_changed {
                        items.push(current.finish(offset));
                        run = Some(PendingRun {
                            start: offset,
                            level,
                            script: char_script,
                        });
                    } else if current.script.is_none() {
                        // Leading neutral prefix adopts the following script.
                        current.script = char_script;
                    }
                }
            }
        }

        if let Some(current) = run {
            items.push(current.finish(text.len()));
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Concatenating item ranges must reconstruct the whole text with no gaps or
    /// overlaps and only UTF-8-boundary slices.
    fn assert_tiles(text: &str, items: &[TextItem]) {
        assert!(!items.is_empty());
        let mut cursor = 0;
        let mut rebuilt = String::new();
        for item in items {
            let range = item.range();
            assert_eq!(range.start, cursor, "gap or overlap at {cursor}");
            assert!(text.is_char_boundary(range.start));
            assert!(text.is_char_boundary(range.end));
            rebuilt.push_str(&text[range.start..range.end]);
            cursor = range.end;
        }
        assert_eq!(cursor, text.len(), "items do not reach end of text");
        assert_eq!(rebuilt, text, "reconstruction mismatch");
    }

    #[test]
    fn pure_latin_is_one_run() {
        let text = "hello world";
        let items = UnicodeTextItemizer::new().itemize(text, TextDirection::LeftToRight);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].range(), TextRange::new(0, text.len()));
        assert_eq!(items[0].script(), ScriptTag::LATIN);
        assert_eq!(items[0].direction(), TextDirection::LeftToRight);
        assert_eq!(items[0].level(), 0);
        assert_tiles(text, &items);
    }

    #[test]
    fn latin_then_cyrillic_splits_at_script_boundary() {
        // Latin "ab" (1 byte each) then Cyrillic "ВГ" (2 bytes each).
        let text = "abВГ";
        let items = UnicodeTextItemizer.itemize(text, TextDirection::LeftToRight);
        assert_eq!(items.len(), 2);

        assert_eq!(items[0].range(), TextRange::new(0, 2));
        assert_eq!(items[0].script(), ScriptTag::LATIN);
        assert_eq!(items[0].direction(), TextDirection::LeftToRight);

        assert_eq!(items[1].range(), TextRange::new(2, 6));
        assert_eq!(items[1].script(), ScriptTag::CYRILLIC);
        assert_eq!(items[1].direction(), TextDirection::LeftToRight);

        assert_tiles(text, &items);
    }

    #[test]
    fn common_space_attaches_to_preceding_run() {
        // "ab гд": the space (Common) attaches to the preceding Latin run.
        let text = "ab гд";
        let items = UnicodeTextItemizer.itemize(text, TextDirection::LeftToRight);
        assert_eq!(items.len(), 2);

        // Latin run includes the trailing space: bytes 0..3 ("ab ").
        assert_eq!(items[0].script(), ScriptTag::LATIN);
        assert_eq!(items[0].range(), TextRange::new(0, 3));
        assert_eq!(&text[0..3], "ab ");

        // Cyrillic "гд" (2 bytes each) fills the remainder.
        assert_eq!(items[1].script(), ScriptTag::CYRILLIC);
        assert_eq!(items[1].range(), TextRange::new(3, text.len()));

        assert_tiles(text, &items);
    }

    #[test]
    fn leading_neutral_prefix_adopts_following_script() {
        // Leading spaces adopt the Latin script of what follows: one run.
        let text = "  ab";
        let items = UnicodeTextItemizer.itemize(text, TextDirection::LeftToRight);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].script(), ScriptTag::LATIN);
        assert_eq!(items[0].range(), TextRange::new(0, text.len()));
        assert_tiles(text, &items);
    }

    #[test]
    fn all_neutral_text_is_common() {
        let text = "123 !!!";
        let items = UnicodeTextItemizer.itemize(text, TextDirection::LeftToRight);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].script(), ScriptTag::COMMON);
        assert_tiles(text, &items);
    }

    #[test]
    fn rtl_hebrew_with_ltr_base_resolves_to_odd_level() {
        // Hebrew is strong RTL: even with an LTR base it resolves to level 1.
        let text = "שלום";
        let items = UnicodeTextItemizer.itemize(text, TextDirection::LeftToRight);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].script(), ScriptTag::HEBREW);
        assert_eq!(items[0].direction(), TextDirection::RightToLeft);
        assert_eq!(items[0].level() % 2, 1);
        assert_eq!(items[0].range(), TextRange::new(0, text.len()));
        assert_tiles(text, &items);
    }

    #[test]
    fn empty_text_matches_single_run_reference() {
        let items = UnicodeTextItemizer.itemize("", TextDirection::LeftToRight);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].range(), TextRange::empty(0));
        assert!(items[0].range().is_empty());
        assert_eq!(items[0].script(), ScriptTag::COMMON);
        assert_eq!(items[0].direction(), TextDirection::LeftToRight);
        assert_eq!(items[0].level(), 0);

        let rtl = UnicodeTextItemizer.itemize("", TextDirection::RightToLeft);
        assert_eq!(rtl[0].direction(), TextDirection::RightToLeft);
        assert_eq!(rtl[0].level(), 1);
    }

    #[test]
    fn is_object_safe() {
        let itemizer: Box<dyn TextItemizer> = Box::new(UnicodeTextItemizer::new());
        let items = itemizer.itemize("ab", TextDirection::LeftToRight);
        assert_eq!(items[0].range(), TextRange::new(0, 2));
    }
}
