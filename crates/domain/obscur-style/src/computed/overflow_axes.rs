#![doc = include_str!("../../docs/api/computed/overflow_axes.doc/_module.md")]

use obscur_identity::InternedSymbol;

use crate::computed::ComputedStyle;
use crate::property::{PropertyId, PropertyValue, layout_keyword};

#[doc = include_str!("../../docs/api/computed/overflow_axes.doc/OverflowAxes.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverflowAxes {
    /// The used value on the inline axis (`overflow-x` in a horizontal writing mode).
    inline: InternedSymbol,
    /// The used value on the block axis (`overflow-y` in a horizontal writing mode).
    block: InternedSymbol,
}

impl OverflowAxes {
    /// Reads both axis longhands out of a node's computed style, treating an absent
    /// declaration (or an absent style) as the initial `visible`, and applies §3's
    /// pairing rule. [`None`] answers "visible on both axes", which is what a node with
    /// no style at all means.
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

    /// Applies css-overflow-3 §3's pairing rule to a specified `(inline, block)` pair:
    /// if exactly one of the two is `visible` or `clip`, that one computes to `auto` or
    /// `hidden` respectively. Exposed so a caller holding the two values already (a
    /// test, or a phase that never had a [`ComputedStyle`](crate::computed::ComputedStyle))
    /// can reach the same answer as [`OverflowAxes::of`].
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

    /// The used `overflow` value on the inline axis.
    #[must_use]
    pub const fn inline_axis(self) -> InternedSymbol {
        self.inline
    }

    /// The used `overflow` value on the block axis — the one the kit can actually
    /// scroll, since a box carries a single scroll offset.
    #[must_use]
    pub const fn block_axis(self) -> InternedSymbol {
        self.block
    }

    /// Whether the box clips its subtree: any axis whose used value is not `visible`.
    /// The kit clips to the whole box rather than to one axis, so one clipping axis is
    /// enough to answer yes.
    #[must_use]
    pub fn clips(self) -> bool {
        self.inline != layout_keyword::OVERFLOW_VISIBLE
            || self.block != layout_keyword::OVERFLOW_VISIBLE
    }

    /// Whether the block axis is `scroll` or `auto` — the two values that offer the
    /// user a scroll (and a scrollbar). `hidden` is excluded: it is scrollable
    /// programmatically but must not answer a wheel.
    #[must_use]
    pub fn block_scrolls(self) -> bool {
        self.block == layout_keyword::OVERFLOW_SCROLL || self.block == layout_keyword::OVERFLOW_AUTO
    }

    /// Whether the box is a **scroll container** in the css-overflow-3 §3 sense:
    /// `scroll`, `auto`, or `hidden` — `hidden` included, because its content "must
    /// still be scrollable programmatically […] and the box is therefore still a scroll
    /// container". Only `visible` and `clip` are excluded; `clip` "forbids scrolling
    /// entirely, through any mechanism".
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
