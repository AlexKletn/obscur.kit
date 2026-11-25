//! A style-local `box-shadow` layer (spec §5.2).

use crate::property::{Length, StyleColor};

#[doc = include_str!("../../docs/api/property/box_shadow.doc/BoxShadow.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadow {
    /// The horizontal offset: positive moves the shadow right.
    pub offset_x: Length,
    /// The vertical offset: positive moves the shadow down.
    pub offset_y: Length,
    /// The blur radius (`0` = a hard edge).
    pub blur: Length,
    /// The spread distance: a drop shadow grows by this, an inset one shrinks.
    pub spread: Length,
    /// The shadow color, or [`None`] for the `currentColor` default.
    pub color: Option<StyleColor>,
    /// Whether this is an `inset` shadow (painted inside the box) rather than a
    /// drop shadow (painted behind it).
    pub inset: bool,
}

impl BoxShadow {
    /// Creates a shadow with the given offsets, blur, spread, optional color, and
    /// `inset` flag.
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

    /// Returns the horizontal offset.
    #[must_use]
    pub const fn offset_x(&self) -> Length {
        self.offset_x
    }

    /// Returns the vertical offset.
    #[must_use]
    pub const fn offset_y(&self) -> Length {
        self.offset_y
    }

    /// Returns the blur radius.
    #[must_use]
    pub const fn blur(&self) -> Length {
        self.blur
    }

    /// Returns the spread distance.
    #[must_use]
    pub const fn spread(&self) -> Length {
        self.spread
    }

    /// Returns the shadow color, or [`None`] for the `currentColor` default.
    #[must_use]
    pub const fn color(&self) -> Option<StyleColor> {
        self.color
    }

    #[doc = include_str!("../../docs/api/property/box_shadow.doc/uses_ch.md")]
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        self.offset_x().is_ch()
            || self.offset_y().is_ch()
            || self.blur().is_ch()
            || self.spread().is_ch()
    }

    /// Returns whether this is an `inset` shadow.
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
