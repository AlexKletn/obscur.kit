#![doc = include_str!("box_shadow.doc/_module.md")]

use crate::property::{Length, StyleColor};

#[doc = include_str!("box_shadow.doc/BoxShadow.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadow {
    #[doc = include_str!("box_shadow.doc/offset_x.md")]
    pub offset_x: Length,
    #[doc = include_str!("box_shadow.doc/offset_y.md")]
    pub offset_y: Length,
    #[doc = include_str!("box_shadow.doc/blur.md")]
    pub blur: Length,
    #[doc = include_str!("box_shadow.doc/spread.md")]
    pub spread: Length,
    #[doc = include_str!("box_shadow.doc/color.md")]
    pub color: Option<StyleColor>,
    #[doc = include_str!("box_shadow.doc/inset.md")]
    pub inset: bool,
}

impl BoxShadow {
    #[doc = include_str!("box_shadow.doc/new.md")]
    #[must_use]
    pub const fn new(
        offset_x: Length,
        offset_y: Length,
        blur: Length,
        spread: Length,
        color: Option<StyleColor>,
        inset: bool,
    ) -> Self {
        Self {
            offset_x,
            offset_y,
            blur,
            spread,
            color,
            inset,
        }
    }

    #[doc = include_str!("box_shadow.doc/offset_x_2.md")]
    #[must_use]
    pub const fn offset_x(&self) -> Length {
        self.offset_x
    }

    #[doc = include_str!("box_shadow.doc/offset_y_2.md")]
    #[must_use]
    pub const fn offset_y(&self) -> Length {
        self.offset_y
    }

    #[doc = include_str!("box_shadow.doc/blur_2.md")]
    #[must_use]
    pub const fn blur(&self) -> Length {
        self.blur
    }

    #[doc = include_str!("box_shadow.doc/spread_2.md")]
    #[must_use]
    pub const fn spread(&self) -> Length {
        self.spread
    }

    #[doc = include_str!("box_shadow.doc/color_2.md")]
    #[must_use]
    pub const fn color(&self) -> Option<StyleColor> {
        self.color
    }

    #[doc = include_str!("box_shadow.doc/uses_ch.md")]
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        self.offset_x().is_ch()
            || self.offset_y().is_ch()
            || self.blur().is_ch()
            || self.spread().is_ch()
    }

    #[doc = include_str!("box_shadow.doc/is_inset.md")]
    #[must_use]
    pub const fn is_inset(&self) -> bool {
        self.inset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_every_field() {
        let shadow = BoxShadow::new(
            Length::px(2.0),
            Length::px(4.0),
            Length::px(6.0),
            Length::px(1.0),
            Some(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
            true,
        );
        assert_eq!(shadow.offset_x(), Length::px(2.0));
        assert_eq!(shadow.offset_y(), Length::px(4.0));
        assert_eq!(shadow.blur(), Length::px(6.0));
        assert_eq!(shadow.spread(), Length::px(1.0));
        assert_eq!(shadow.color(), Some(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        assert!(shadow.is_inset());
    }

    #[test]
    fn absent_color_is_current_color_default() {
        let shadow = BoxShadow::new(
            Length::px(0.0),
            Length::px(0.0),
            Length::px(0.0),
            Length::px(0.0),
            None,
            false,
        );
        assert_eq!(shadow.color(), None);
        assert!(!shadow.is_inset());
    }
}
