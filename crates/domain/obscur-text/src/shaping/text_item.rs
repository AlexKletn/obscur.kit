//! One itemized text segment: a uniform script + direction run (spec §5.10).

use crate::shaping::{ScriptTag, TextDirection};
use crate::span::TextRange;

#[doc = include_str!("../../docs/api/shaping/text_item.doc/TextItem.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextItem {
    /// The byte range of the source text this item covers.
    range: TextRange,
    /// The script the run is written in.
    script: ScriptTag,
    /// The resolved base direction of the run.
    direction: TextDirection,
    /// The resolved bidi embedding level (even = LTR, odd = RTL).
    level: u8,
}

impl TextItem {
    /// Creates an item covering `range`, written in `script`, laid out in
    /// `direction` at bidi embedding `level`.
    #[must_use]
    pub const fn new(
        range: TextRange,
        script: ScriptTag,
        direction: TextDirection,
        level: u8,
    ) -> Self {
        Self {
            range,
            script,
            direction,
            level,
        }
    }

    /// Returns the byte range of the source text this item covers.
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    /// Returns the script the run is written in.
    #[must_use]
    pub const fn script(&self) -> ScriptTag {
        self.script
    }

    /// Returns the resolved base direction of the run.
    #[must_use]
    pub const fn direction(&self) -> TextDirection {
        self.direction
    }

    /// Returns the resolved bidi embedding level (even = LTR, odd = RTL).
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_report_construction() {
        let item = TextItem::new(
            TextRange::new(0, 5),
            ScriptTag::LATIN,
            TextDirection::LeftToRight,
            0,
        );
        assert_eq!(item.range(), TextRange::new(0, 5));
        assert_eq!(item.script(), ScriptTag::LATIN);
        assert_eq!(item.direction(), TextDirection::LeftToRight);
        assert_eq!(item.level(), 0);
    }

    #[test]
    fn rtl_item_carries_odd_level() {
        let item = TextItem::new(
            TextRange::new(0, 4),
            ScriptTag::ARABIC,
            TextDirection::RightToLeft,
            1,
        );
        assert_eq!(item.direction(), TextDirection::RightToLeft);
        assert_eq!(item.level(), 1);
    }
}
