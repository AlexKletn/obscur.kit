//! The output of the measure phase for a node.

use obscur_geometry::Size;

/// The result of the **measure** phase for a node: the size it resolved to
/// under the constraints it was measured against (spec §4.2).
///
/// The size is expressed in fractional logical pixels; no rounding is applied
/// (spec §4.6).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutResult {
    size: Size,
}

impl LayoutResult {
    /// Creates a measure result carrying the resolved `size`.
    #[must_use]
    pub const fn new(size: Size) -> Self {
        Self { size }
    }

    /// The resolved size the node wants.
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
