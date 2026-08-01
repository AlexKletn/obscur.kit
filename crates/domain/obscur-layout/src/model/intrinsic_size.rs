#![doc = include_str!("intrinsic_size.doc/_module.md")]

use obscur_geometry::Size;

#[doc = include_str!("intrinsic_size.doc/IntrinsicSize.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntrinsicSize {
    min_content: Size,
    max_content: Size,
}

impl IntrinsicSize {
    #[doc = include_str!("intrinsic_size.doc/new.md")]
    #[must_use]
    pub const fn new(min_content: Size, max_content: Size) -> Self {
        Self {
            min_content,
            max_content,
        }
    }

    #[doc = include_str!("intrinsic_size.doc/fixed.md")]
    #[must_use]
    pub const fn fixed(size: Size) -> Self {
        Self {
            min_content: size,
            max_content: size,
        }
    }

    #[doc = include_str!("intrinsic_size.doc/min_content.md")]
    #[must_use]
    pub const fn min_content(self) -> Size {
        self.min_content
    }

    #[doc = include_str!("intrinsic_size.doc/max_content.md")]
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
