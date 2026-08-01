#![doc = include_str!("layout_pass_result.doc/_module.md")]

use crate::model::LayoutBox;

#[doc = include_str!("layout_pass_result.doc/LayoutPassResult.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutPassResult {
    #[doc = include_str!("layout_pass_result.doc/layout_box.md")]
    layout_box: LayoutBox,
    #[doc = include_str!("layout_pass_result.doc/geometry_changed.md")]
    geometry_changed: bool,
}

impl LayoutPassResult {
    #[doc = include_str!("layout_pass_result.doc/new.md")]
    #[must_use]
    pub const fn new(layout_box: LayoutBox, geometry_changed: bool) -> Self {
        Self {
            layout_box,
            geometry_changed,
        }
    }

    #[doc = include_str!("layout_pass_result.doc/unchanged.md")]
    #[must_use]
    pub const fn unchanged(layout_box: LayoutBox) -> Self {
        Self::new(layout_box, false)
    }

    #[doc = include_str!("layout_pass_result.doc/changed.md")]
    #[must_use]
    pub const fn changed(layout_box: LayoutBox) -> Self {
        Self::new(layout_box, true)
    }

    #[doc = include_str!("layout_pass_result.doc/layout_box_2.md")]
    #[must_use]
    pub const fn layout_box(self) -> LayoutBox {
        self.layout_box
    }

    #[doc = include_str!("layout_pass_result.doc/geometry_changed_2.md")]
    #[must_use]
    pub const fn geometry_changed(self) -> bool {
        self.geometry_changed
    }

    #[doc = include_str!("layout_pass_result.doc/propagates_upward.md")]
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
