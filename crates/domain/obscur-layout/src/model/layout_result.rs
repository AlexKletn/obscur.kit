#![doc = include_str!("layout_result.doc/_module.md")]

use obscur_geometry::Size;

#[doc = include_str!("layout_result.doc/LayoutResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutResult {
    size: Size,
}

impl LayoutResult {
    #[doc = include_str!("layout_result.doc/new.md")]
    #[must_use]
    pub const fn new(size: Size) -> Self {
        Self { size }
    }

    #[doc = include_str!("layout_result.doc/size.md")]
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutResult;
    use obscur_geometry::Size;

    #[test]
    fn new_exposes_size() {
        let result = LayoutResult::new(Size::new(120.0, 24.0));
        assert_eq!(result.size(), Size::new(120.0, 24.0));
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(LayoutResult::default().size(), Size::ZERO);
    }
}
