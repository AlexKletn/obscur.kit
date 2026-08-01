#![doc = include_str!("filter_op.doc/_module.md")]

use crate::property::BoxShadow;

#[doc = include_str!("filter_op.doc/FilterOp.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOp {
    #[doc = include_str!("filter_op.doc/DropShadow.md")]
    DropShadow(BoxShadow),
    #[doc = include_str!("filter_op.doc/Blur.md")]
    Blur(f32),
    #[doc = include_str!("filter_op.doc/Brightness.md")]
    Brightness(f32),
    #[doc = include_str!("filter_op.doc/Contrast.md")]
    Contrast(f32),
    #[doc = include_str!("filter_op.doc/Saturate.md")]
    Saturate(f32),
    #[doc = include_str!("filter_op.doc/Grayscale.md")]
    Grayscale(f32),
    #[doc = include_str!("filter_op.doc/Sepia.md")]
    Sepia(f32),
    #[doc = include_str!("filter_op.doc/Invert.md")]
    Invert(f32),
    #[doc = include_str!("filter_op.doc/HueRotate.md")]
    HueRotate(f32),
    #[doc = include_str!("filter_op.doc/Opacity.md")]
    Opacity(f32),
}

impl FilterOp {
    #[doc = include_str!("filter_op.doc/as_drop_shadow.md")]
    #[must_use]
    pub const fn as_drop_shadow(&self) -> Option<&BoxShadow> {
        match self {
            Self::DropShadow(shadow) => Some(shadow),
            _ => None,
        }
    }

    #[doc = include_str!("filter_op.doc/as_blur.md")]
    #[must_use]
    pub const fn as_blur(&self) -> Option<f32> {
        match self {
            Self::Blur(radius) => Some(*radius),
            _ => None,
        }
    }

    #[doc = include_str!("filter_op.doc/is_color_op.md")]
    #[must_use]
    pub const fn is_color_op(&self) -> bool {
        !matches!(self, Self::DropShadow(_) | Self::Blur(_))
    }

    #[doc = include_str!("filter_op.doc/uses_ch.md")]
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        match self.as_drop_shadow() {
            Some(shadow) => shadow.uses_ch(),
            None => false,
        }
    }

    #[doc = include_str!("filter_op.doc/is_drop_shadow.md")]
    #[must_use]
    pub const fn is_drop_shadow(&self) -> bool {
        matches!(self, Self::DropShadow(_))
    }

    #[doc = include_str!("filter_op.doc/is_blur.md")]
    #[must_use]
    pub const fn is_blur(&self) -> bool {
        matches!(self, Self::Blur(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::{Length, StyleColor};

    fn a_drop_shadow() -> BoxShadow {
        BoxShadow::new(
            Length::px(1.0),
            Length::px(2.0),
            Length::px(3.0),
            Length::px(0.0),
            Some(StyleColor::BLACK),
            false,
        )
    }

    #[test]
    fn drop_shadow_accessors() {
        let op = FilterOp::DropShadow(a_drop_shadow());
        assert!(op.is_drop_shadow());
        assert!(!op.is_blur());
        assert_eq!(op.as_drop_shadow(), Some(&a_drop_shadow()));
        assert_eq!(op.as_blur(), None);
    }

    #[test]
    fn blur_accessors() {
        let op = FilterOp::Blur(8.0);
        assert!(op.is_blur());
        assert!(!op.is_drop_shadow());
        assert_eq!(op.as_blur(), Some(8.0));
        assert_eq!(op.as_drop_shadow(), None);
        assert!(!op.is_color_op());
    }

    #[test]
    fn color_ops_are_classified() {
        // Spatial ops are NOT color ops.
        assert!(!FilterOp::Blur(2.0).is_color_op());
        assert!(!FilterOp::DropShadow(a_drop_shadow()).is_color_op());
        // Every color-matrix function IS a color op (folded into one 4×5 matrix).
        for op in [
            FilterOp::Brightness(1.2),
            FilterOp::Contrast(0.8),
            FilterOp::Saturate(2.0),
            FilterOp::Grayscale(1.0),
            FilterOp::Sepia(1.0),
            FilterOp::Invert(1.0),
            FilterOp::HueRotate(1.5),
            FilterOp::Opacity(0.5),
        ] {
            assert!(op.is_color_op(), "{op:?} is a color op");
            assert_eq!(op.as_blur(), None);
            assert_eq!(op.as_drop_shadow(), None);
        }
    }
}
