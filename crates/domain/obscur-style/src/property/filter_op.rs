//! A single CSS `filter` function op (spec §5.2).

use crate::property::BoxShadow;

/// A single ordered `filter` function op, as authored in a stylesheet.
///
/// The `filter` property is a `<filter-function-list>` applied in author order.
/// This enum models the two functions the pipeline can honor today, preserving
/// that order:
///
/// - [`FilterOp::DropShadow`] — a `drop-shadow(...)`: a `box-shadow` DROP subset
///   (`spread == 0`, `inset == false`), painted behind the element like a drop
///   `box-shadow` (spec §5.2). The coordinator emits it as a `DrawShadow`.
/// - [`FilterOp::Blur`] — a `blur(<length>)` content blur; the radius is resolved
///   to logical pixels at lower time. The coordinator brackets the element's
///   subtree with a `PushLayer(Blur)` … `PopLayer`, and the Vello renderer runs a
///   real separable gaussian pass over the composited subtree.
///
/// Other filter functions (`brightness()`, `contrast()`, color-matrix, …) have no
/// GPU-expressible path yet and are dropped at CSS lower time (documented), so
/// this enum stays small and extensible (color-matrix / `backdrop-filter` are the
/// natural next variants).
///
/// Because [`BoxShadow`] and the blur radius carry `f32` data, this type derives
/// [`PartialEq`] but not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOp {
    /// A `drop-shadow(...)` op: a drop-only [`BoxShadow`].
    DropShadow(BoxShadow),
    /// A `blur(<length>)` content blur; the radius is in logical pixels (`>= 0`).
    Blur(f32),
}

impl FilterOp {
    /// Returns the drop-shadow if this is a [`FilterOp::DropShadow`].
    #[must_use]
    pub const fn as_drop_shadow(&self) -> Option<&BoxShadow> {
        match self {
            Self::DropShadow(shadow) => Some(shadow),
            Self::Blur(_) => None,
        }
    }

    /// Returns the blur radius (logical px) if this is a [`FilterOp::Blur`].
    #[must_use]
    pub const fn as_blur(&self) -> Option<f32> {
        match self {
            Self::Blur(radius) => Some(*radius),
            Self::DropShadow(_) => None,
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
    }
}
