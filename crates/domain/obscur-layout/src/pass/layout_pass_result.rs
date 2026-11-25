//! The result of running measure + arrange for a node or subtree.

use crate::model::LayoutBox;

#[doc = include_str!("../../docs/api/pass/layout_pass_result.doc/LayoutPassResult.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutPassResult {
    /// The arranged box produced by this pass.
    layout_box: LayoutBox,
    /// Whether the produced geometry differs from the previous pass.
    geometry_changed: bool,
}

impl LayoutPassResult {
    /// Creates a result carrying the arranged `layout_box` and whether the
    /// geometry changed from the previous pass.
    #[must_use]
    pub const fn new(layout_box: LayoutBox, geometry_changed: bool) -> Self {
        Self {
            layout_box,
            geometry_changed,
        }
    }

    /// A result whose geometry is unchanged from the previous pass.
    #[must_use]
    pub const fn unchanged(layout_box: LayoutBox) -> Self {
        Self::new(layout_box, false)
    }

    /// A result whose geometry changed from the previous pass.
    #[must_use]
    pub const fn changed(layout_box: LayoutBox) -> Self {
        Self::new(layout_box, true)
    }

    /// The arranged box produced by this pass.
    #[must_use]
    pub const fn layout_box(self) -> LayoutBox {
        self.layout_box
    }

    /// Whether the produced geometry differs from the previous pass.
    #[must_use]
    pub const fn geometry_changed(self) -> bool {
        self.geometry_changed
    }

    /// Whether this result should propagate invalidation **upward** to
    /// ancestors (spec §4.3: propagate up only when output geometry changes).
    #[must_use]
    pub const fn propagates_upward(self) -> bool {
        self.geometry_changed
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutPassResult;
    use crate::model::LayoutBox;
    use obscur_geometry::Size;

    fn a_box() -> LayoutBox {
        LayoutBox::from_size(Size::new(10.0, 10.0))
    }

    #[test]
    fn unchanged_does_not_propagate() {
        let r = LayoutPassResult::unchanged(a_box());
        assert!(!r.geometry_changed());
        assert!(!r.propagates_upward());
        assert_eq!(r.layout_box(), a_box());
    }

    #[test]
    fn changed_propagates_upward() {
        let r = LayoutPassResult::changed(a_box());
        assert!(r.geometry_changed());
        assert!(r.propagates_upward());
    }

    #[test]
    fn new_carries_flag() {
        assert!(LayoutPassResult::new(a_box(), true).propagates_upward());
        assert!(!LayoutPassResult::new(a_box(), false).propagates_upward());
    }
}
