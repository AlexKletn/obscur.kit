#![doc = include_str!("logical_property.doc/_module.md")]

use crate::property::{PropertyId, WritingMode};

impl PropertyId {
    #[doc = include_str!("logical_property.doc/physical_under.md")]
    #[must_use]
    pub const fn physical_under(self, rtl: bool, wm: WritingMode) -> Option<Self> {
        let vertical = wm.is_vertical();
        Some(match self {
            // Inline axis: flipped by `direction`, and rotated to vertical by a
            // vertical `writing-mode` (line-left = physical top).
            Self::MarginInlineStart => match (vertical, rtl) {
                (false, false) => Self::MarginLeft,
                (false, true) => Self::MarginRight,
                (true, false) => Self::MarginTop,
                (true, true) => Self::MarginBottom,
            },
            Self::MarginInlineEnd => match (vertical, rtl) {
                (false, false) => Self::MarginRight,
                (false, true) => Self::MarginLeft,
                (true, false) => Self::MarginBottom,
                (true, true) => Self::MarginTop,
            },
            Self::PaddingInlineStart => match (vertical, rtl) {
                (false, false) => Self::PaddingLeft,
                (false, true) => Self::PaddingRight,
                (true, false) => Self::PaddingTop,
                (true, true) => Self::PaddingBottom,
            },
            Self::PaddingInlineEnd => match (vertical, rtl) {
                (false, false) => Self::PaddingRight,
                (false, true) => Self::PaddingLeft,
                (true, false) => Self::PaddingBottom,
                (true, true) => Self::PaddingTop,
            },
            Self::InsetInlineStart => match (vertical, rtl) {
                (false, false) => Self::Left,
                (false, true) => Self::Right,
                (true, false) => Self::Top,
                (true, true) => Self::Bottom,
            },
            Self::InsetInlineEnd => match (vertical, rtl) {
                (false, false) => Self::Right,
                (false, true) => Self::Left,
                (true, false) => Self::Bottom,
                (true, true) => Self::Top,
            },
            Self::BorderInlineStartWidth => match (vertical, rtl) {
                (false, false) => Self::BorderLeftWidth,
                (false, true) => Self::BorderRightWidth,
                (true, false) => Self::BorderTopWidth,
                (true, true) => Self::BorderBottomWidth,
            },
            Self::BorderInlineEndWidth => match (vertical, rtl) {
                (false, false) => Self::BorderRightWidth,
                (false, true) => Self::BorderLeftWidth,
                (true, false) => Self::BorderBottomWidth,
                (true, true) => Self::BorderTopWidth,
            },
            Self::BorderInlineStartStyle => match (vertical, rtl) {
                (false, false) => Self::BorderLeftStyle,
                (false, true) => Self::BorderRightStyle,
                (true, false) => Self::BorderTopStyle,
                (true, true) => Self::BorderBottomStyle,
            },
            Self::BorderInlineEndStyle => match (vertical, rtl) {
                (false, false) => Self::BorderRightStyle,
                (false, true) => Self::BorderLeftStyle,
                (true, false) => Self::BorderBottomStyle,
                (true, true) => Self::BorderTopStyle,
            },
            // Block axis: vertical under `horizontal-tb` (top/bottom), horizontal
            // under the vertical modes — `block-start` is the right under
            // `vertical-rl`, the left under `vertical-lr` (§6.2). `direction` does
            // not touch it.
            Self::MarginBlockStart => match wm {
                WritingMode::HorizontalTb => Self::MarginTop,
                WritingMode::VerticalRl => Self::MarginRight,
                WritingMode::VerticalLr => Self::MarginLeft,
            },
            Self::MarginBlockEnd => match wm {
                WritingMode::HorizontalTb => Self::MarginBottom,
                WritingMode::VerticalRl => Self::MarginLeft,
                WritingMode::VerticalLr => Self::MarginRight,
            },
            Self::PaddingBlockStart => match wm {
                WritingMode::HorizontalTb => Self::PaddingTop,
                WritingMode::VerticalRl => Self::PaddingRight,
                WritingMode::VerticalLr => Self::PaddingLeft,
            },
            Self::PaddingBlockEnd => match wm {
                WritingMode::HorizontalTb => Self::PaddingBottom,
                WritingMode::VerticalRl => Self::PaddingLeft,
                WritingMode::VerticalLr => Self::PaddingRight,
            },
            Self::InsetBlockStart => match wm {
                WritingMode::HorizontalTb => Self::Top,
                WritingMode::VerticalRl => Self::Right,
                WritingMode::VerticalLr => Self::Left,
            },
            Self::InsetBlockEnd => match wm {
                WritingMode::HorizontalTb => Self::Bottom,
                WritingMode::VerticalRl => Self::Left,
                WritingMode::VerticalLr => Self::Right,
            },
            Self::BorderBlockStartWidth => match wm {
                WritingMode::HorizontalTb => Self::BorderTopWidth,
                WritingMode::VerticalRl => Self::BorderRightWidth,
                WritingMode::VerticalLr => Self::BorderLeftWidth,
            },
            Self::BorderBlockEndWidth => match wm {
                WritingMode::HorizontalTb => Self::BorderBottomWidth,
                WritingMode::VerticalRl => Self::BorderLeftWidth,
                WritingMode::VerticalLr => Self::BorderRightWidth,
            },
            Self::BorderBlockStartStyle => match wm {
                WritingMode::HorizontalTb => Self::BorderTopStyle,
                WritingMode::VerticalRl => Self::BorderRightStyle,
                WritingMode::VerticalLr => Self::BorderLeftStyle,
            },
            Self::BorderBlockEndStyle => match wm {
                WritingMode::HorizontalTb => Self::BorderBottomStyle,
                WritingMode::VerticalRl => Self::BorderLeftStyle,
                WritingMode::VerticalLr => Self::BorderRightStyle,
            },
            _ => return None,
        })
    }

    #[doc = include_str!("logical_property.doc/is_logical.md")]
    #[must_use]
    pub const fn is_logical(self) -> bool {
        // The direction / writing-mode are arbitrary here: a logical property maps
        // under any of them, so membership is what this asks.
        self.physical_under(false, WritingMode::HorizontalTb)
            .is_some()
    }

    #[doc = include_str!("logical_property.doc/LOGICAL.md")]
    pub const LOGICAL: [Self; 20] = [
        Self::MarginInlineStart,
        Self::MarginInlineEnd,
        Self::MarginBlockStart,
        Self::MarginBlockEnd,
        Self::PaddingInlineStart,
        Self::PaddingInlineEnd,
        Self::PaddingBlockStart,
        Self::PaddingBlockEnd,
        Self::InsetInlineStart,
        Self::InsetInlineEnd,
        Self::InsetBlockStart,
        Self::InsetBlockEnd,
        Self::BorderInlineStartWidth,
        Self::BorderInlineEndWidth,
        Self::BorderBlockStartWidth,
        Self::BorderBlockEndWidth,
        Self::BorderInlineStartStyle,
        Self::BorderInlineEndStyle,
        Self::BorderBlockStartStyle,
        Self::BorderBlockEndStyle,
    ];
}
