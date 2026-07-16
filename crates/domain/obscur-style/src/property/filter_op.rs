//! A single CSS `filter` function op (spec §5.2).

use crate::property::BoxShadow;

/// A single ordered `filter` function op, as authored in a stylesheet.
///
/// The `filter` property is a `<filter-function-list>` applied in author order.
/// This enum models the functions the pipeline can honor today, preserving that
/// order:
///
/// - [`FilterOp::DropShadow`] — a `drop-shadow(...)`: a `box-shadow` DROP subset
///   (`spread == 0`, `inset == false`), painted behind the element like a drop
///   `box-shadow` (spec §5.2). The coordinator emits it as a `DrawShadow`.
/// - [`FilterOp::Blur`] — a `blur(<length>)` content blur; the radius is resolved
///   to logical pixels at lower time. The coordinator brackets the element's
///   subtree with a `PushLayer(Blur)` … `PopLayer`, and the Vello renderer runs a
///   real separable gaussian pass over the composited subtree.
/// - The **color-matrix** ops — [`Brightness`](FilterOp::Brightness),
///   [`Contrast`](FilterOp::Contrast), [`Saturate`](FilterOp::Saturate),
///   [`Grayscale`](FilterOp::Grayscale), [`Sepia`](FilterOp::Sepia),
///   [`Invert`](FilterOp::Invert), [`HueRotate`](FilterOp::HueRotate),
///   [`Opacity`](FilterOp::Opacity) — each an affine color transform (CSS Filter
///   Effects §). Because every one is affine, the coordinator folds a whole
///   color-op chain into a single 4×5 matrix applied in one owned wgpu pass over
///   Vello's output (Vello 0.9 has no color filters).
///
/// `drop-shadow` (a spatial shadow) and `blur` (a spatial gaussian) are NOT color
/// ops and stay their own passes; a filter list may freely mix them.
///
/// Because [`BoxShadow`] and the numeric amounts carry `f32` data, this type
/// derives [`PartialEq`] but not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOp {
    /// A `drop-shadow(...)` op: a drop-only [`BoxShadow`].
    DropShadow(BoxShadow),
    /// A `blur(<length>)` content blur; the radius is in logical pixels (`>= 0`).
    Blur(f32),
    /// A `brightness(<number|percentage>)`: `RGB *= amount` (amount `>= 0`).
    Brightness(f32),
    /// A `contrast(<number|percentage>)`: `RGB' = (RGB − 0.5)·amount + 0.5`.
    Contrast(f32),
    /// A `saturate(<number|percentage>)`: the spec luminance saturation matrix.
    Saturate(f32),
    /// A `grayscale(<number|percentage>)`: interpolate toward the luma matrix
    /// (amount clamped `0..1`; `1` is fully desaturated).
    Grayscale(f32),
    /// A `sepia(<number|percentage>)`: the spec sepia matrix (amount `0..1`).
    Sepia(f32),
    /// An `invert(<number|percentage>)`: `channel' = (1 − 2·amount)·channel +
    /// amount` per RGB (amount clamped `0..1`).
    Invert(f32),
    /// A `hue-rotate(<angle>)`: the spec hue-rotation matrix; the angle is stored
    /// in **radians**.
    HueRotate(f32),
    /// An `opacity(<number|percentage>)`: `A *= amount` (amount clamped `0..1`).
    Opacity(f32),
}

impl FilterOp {
    /// Returns the drop-shadow if this is a [`FilterOp::DropShadow`].
    #[must_use]
    pub const fn as_drop_shadow(&self) -> Option<&BoxShadow> {
        match self {
            Self::DropShadow(shadow) => Some(shadow),
            _ => None,
        }
    }

    /// Returns the blur radius (logical px) if this is a [`FilterOp::Blur`].
    #[must_use]
    pub const fn as_blur(&self) -> Option<f32> {
        match self {
            Self::Blur(radius) => Some(*radius),
            _ => None,
        }
    }

    /// Whether this op is an affine **color** transform (one of the color-matrix
    /// functions the coordinator folds into a single 4×5 matrix) — i.e. anything
    /// that is neither a [`DropShadow`](FilterOp::DropShadow) nor a
    /// [`Blur`](FilterOp::Blur) (both spatial, not color).
    #[must_use]
    pub const fn is_color_op(&self) -> bool {
        !matches!(self, Self::DropShadow(_) | Self::Blur(_))
    }

    /// Whether this op carries a `ch` term.
    ///
    /// Only `drop-shadow()` carries lengths at all: `blur(<length>)`'s radius is
    /// already an `f32` by computed-value time (the CSS lowering resolved it), so
    /// a `blur(1ch)` is lost before this is asked — a lowering-side limitation
    /// mirrored in `absolutize_filter`, not papered over here.
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        match self.as_drop_shadow() {
            Some(shadow) => shadow.uses_ch(),
            None => false,
        }
    }

    /// Whether this is a [`FilterOp::DropShadow`].
    #[must_use]
    pub const fn is_drop_shadow(&self) -> bool {
        matches!(self, Self::DropShadow(_))
    }

    /// Whether this is a [`FilterOp::Blur`].
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
