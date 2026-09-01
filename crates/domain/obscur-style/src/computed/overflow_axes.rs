#![doc = include_str!("overflow_axes.doc/_module.md")]

use obscur_identity::InternedSymbol;

use crate::computed::ComputedStyle;
use crate::property::{PropertyId, PropertyValue, layout_keyword};

#[doc = include_str!("overflow_axes.doc/OverflowAxes.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverflowAxes {
    #[doc = include_str!("overflow_axes.doc/inline.md")]
    inline: InternedSymbol,
    #[doc = include_str!("overflow_axes.doc/block.md")]
    block: InternedSymbol,
}

impl OverflowAxes {
    #[doc = include_str!("overflow_axes.doc/of.md")]
    #[must_use]
    pub fn of(style: Option<&ComputedStyle>) -> Self {
        let axis = |id: PropertyId| match style.map(|s| s.get(id)) {
            Some(Some(value)) => match value.value() {
                PropertyValue::Keyword(symbol) => *symbol,
                _ => layout_keyword::OVERFLOW_VISIBLE,
            },
            _ => layout_keyword::OVERFLOW_VISIBLE,
        };
        Self::computed(axis(PropertyId::OverflowX), axis(PropertyId::OverflowY))
    }

    #[doc = include_str!("overflow_axes.doc/computed.md")]
    #[must_use]
    pub fn computed(inline: InternedSymbol, block: InternedSymbol) -> Self {
        // css-overflow-3 §3: an author who clips or scrolls ONE axis has said
        // something about the other, because a box cannot let content spill out
        // of an axis it also has to scroll. The spec spends that observation
        // here: `visible` beside a non-`visible` sibling becomes `auto`, `clip`
        // beside one becomes `hidden`.
        let loose = |value: InternedSymbol| {
            value == layout_keyword::OVERFLOW_VISIBLE || value == layout_keyword::OVERFLOW_CLIP
        };
        let paired = |value: InternedSymbol| {
            if value == layout_keyword::OVERFLOW_VISIBLE {
                layout_keyword::OVERFLOW_AUTO
            } else {
                layout_keyword::OVERFLOW_HIDDEN
            }
        };
        match (loose(inline), loose(block)) {
            (true, false) => Self {
                inline: paired(inline),
                block,
            },
            (false, true) => Self {
                inline,
                block: paired(block),
            },
            _ => Self { inline, block },
        }
    }

    #[doc = include_str!("overflow_axes.doc/inline_axis.md")]
    #[must_use]
    pub const fn inline_axis(self) -> InternedSymbol {
        self.inline
    }

    #[doc = include_str!("overflow_axes.doc/block_axis.md")]
    #[must_use]
    pub const fn block_axis(self) -> InternedSymbol {
        self.block
    }

    #[doc = include_str!("overflow_axes.doc/clips.md")]
    #[must_use]
    pub fn clips(self) -> bool {
        self.inline != layout_keyword::OVERFLOW_VISIBLE
            || self.block != layout_keyword::OVERFLOW_VISIBLE
    }

    #[doc = include_str!("overflow_axes.doc/block_scrolls.md")]
    #[must_use]
    pub fn block_scrolls(self) -> bool {
        self.block == layout_keyword::OVERFLOW_SCROLL || self.block == layout_keyword::OVERFLOW_AUTO
    }

    #[doc = include_str!("overflow_axes.doc/establishes_scroll_container.md")]
    #[must_use]
    pub fn establishes_scroll_container(self) -> bool {
        // Every axis pair that scrolls at all leaves the block axis outside
        // `visible` / `clip` — that is what `computed` above guarantees — so the
        // box-level question is answered by one axis.
        self.block != layout_keyword::OVERFLOW_VISIBLE
            && self.block != layout_keyword::OVERFLOW_CLIP
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computed::ComputedValue;

    fn style(pairs: &[(PropertyId, InternedSymbol)]) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        for (id, symbol) in pairs {
            style.set(*id, ComputedValue::keyword(*symbol));
        }
        style
    }

    #[test]
    fn an_absent_declaration_is_visible_on_both_axes() {
        let axes = OverflowAxes::of(None);
        assert_eq!(axes.inline_axis(), layout_keyword::OVERFLOW_VISIBLE);
        assert_eq!(axes.block_axis(), layout_keyword::OVERFLOW_VISIBLE);
        assert!(!axes.clips());
        assert!(!axes.block_scrolls());
        assert!(!axes.establishes_scroll_container());
    }

    #[test]
    fn one_axis_alone_is_read_as_that_axis() {
        let declared = style(&[(PropertyId::OverflowY, layout_keyword::OVERFLOW_AUTO)]);
        let axes = OverflowAxes::of(Some(&declared));
        assert!(axes.block_scrolls());
        assert!(axes.establishes_scroll_container());
        assert!(axes.clips());
    }

    #[test]
    fn a_scrolling_axis_pulls_a_visible_sibling_to_auto() {
        // css-overflow-3 §3: `overflow-x: hidden` beside an unstated (`visible`)
        // block axis computes that axis to `auto`, which is why a box with only
        // the inline axis clipped still scrolls vertically in every browser.
        let declared = style(&[(PropertyId::OverflowX, layout_keyword::OVERFLOW_HIDDEN)]);
        let axes = OverflowAxes::of(Some(&declared));
        assert_eq!(axes.block_axis(), layout_keyword::OVERFLOW_AUTO);
        assert!(axes.block_scrolls());
    }

    #[test]
    fn clip_beside_a_scrolling_axis_becomes_hidden() {
        let axes = OverflowAxes::computed(
            layout_keyword::OVERFLOW_CLIP,
            layout_keyword::OVERFLOW_SCROLL,
        );
        assert_eq!(axes.inline_axis(), layout_keyword::OVERFLOW_HIDDEN);
        assert_eq!(axes.block_axis(), layout_keyword::OVERFLOW_SCROLL);
    }

    #[test]
    fn clip_on_both_axes_is_not_a_scroll_container_but_still_clips() {
        let axes =
            OverflowAxes::computed(layout_keyword::OVERFLOW_CLIP, layout_keyword::OVERFLOW_CLIP);
        assert!(axes.clips());
        assert!(!axes.establishes_scroll_container());
        assert!(!axes.block_scrolls());
    }

    #[test]
    fn hidden_is_a_scroll_container_that_the_user_cannot_scroll() {
        let axes = OverflowAxes::computed(
            layout_keyword::OVERFLOW_HIDDEN,
            layout_keyword::OVERFLOW_HIDDEN,
        );
        assert!(axes.establishes_scroll_container());
        assert!(!axes.block_scrolls());
    }
}
