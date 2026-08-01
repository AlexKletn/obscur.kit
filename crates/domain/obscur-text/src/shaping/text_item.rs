#![doc = include_str!("text_item.doc/_module.md")]

use crate::shaping::{ScriptTag, TextDirection};
use crate::span::TextRange;

#[doc = include_str!("text_item.doc/TextItem.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextItem {
    #[doc = include_str!("text_item.doc/range.md")]
    range: TextRange,
    #[doc = include_str!("text_item.doc/script.md")]
    script: ScriptTag,
    #[doc = include_str!("text_item.doc/direction.md")]
    direction: TextDirection,
    #[doc = include_str!("text_item.doc/level.md")]
    level: u8,
}

impl TextItem {
    #[doc = include_str!("text_item.doc/new.md")]
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

    #[doc = include_str!("text_item.doc/range_2.md")]
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    #[doc = include_str!("text_item.doc/script_2.md")]
    #[must_use]
    pub const fn script(&self) -> ScriptTag {
        self.script
    }

    #[doc = include_str!("text_item.doc/direction_2.md")]
    #[must_use]
    pub const fn direction(&self) -> TextDirection {
        self.direction
    }

    #[doc = include_str!("text_item.doc/level_2.md")]
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
