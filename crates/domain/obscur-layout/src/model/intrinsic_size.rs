//! A node's intrinsic (content-driven) sizes.

use obscur_geometry::Size;

/// The intrinsic sizes a node's content wants, independent of any external
/// constraints (spec §4.1).
///
/// * `min_content` is the smallest size the content can take without overflowing
///   (e.g. the longest unbreakable word).
/// * `max_content` is the largest size the content wants when given unbounded
///   space (e.g. a paragraph laid out on a single line).
///
/// Sizes are fractional logical pixels; no rounding is applied (spec §4.6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntrinsicSize {
    min_content: Size,
    max_content: Size,
}

impl IntrinsicSize {
    /// Creates intrinsic sizes from an explicit minimum- and maximum-content size.
    #[must_use]
    pub const fn new(min_content: Size, max_content: Size) -> Self {
        Self {
            min_content,
            max_content,
        }
    }

    /// Creates intrinsic sizes for content that has a single fixed size, where
    /// the minimum- and maximum-content sizes are equal.
    #[must_use]
    pub const fn fixed(size: Size) -> Self {
        Self {
            min_content: size,
            max_content: size,
        }
    }

    /// The minimum-content size.
    #[must_use]
    pub const fn min_content(self) -> Size {
        self.min_content
    }

    /// The maximum-content size.
    #[must_use]
    pub const fn max_content(self) -> Size {
        self.max_content
    }
}

#[cfg(test)]
mod tests {
    use super::IntrinsicSize;
    use obscur_geometry::Size;

    #[test]
    fn new_stores_both_sizes() {
        let intrinsic = IntrinsicSize::new(Size::new(10.0, 5.0), Size::new(40.0, 5.0));
        assert_eq!(intrinsic.min_content(), Size::new(10.0, 5.0));
        assert_eq!(intrinsic.max_content(), Size::new(40.0, 5.0));
    }

    #[test]
    fn fixed_sets_equal_bounds() {
        let intrinsic = IntrinsicSize::fixed(Size::new(7.5, 3.0));
        assert_eq!(intrinsic.min_content(), intrinsic.max_content());
        assert_eq!(intrinsic.min_content(), Size::new(7.5, 3.0));
    }
}
