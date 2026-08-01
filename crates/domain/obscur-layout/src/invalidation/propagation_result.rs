#![doc = include_str!("propagation_result.doc/_module.md")]

use crate::pass::LayoutPassResult;

#[doc = include_str!("propagation_result.doc/PropagationResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropagationResult {
    #[doc = include_str!("propagation_result.doc/geometry_changed.md")]
    geometry_changed: bool,
    #[doc = include_str!("propagation_result.doc/contained.md")]
    contained: bool,
}

impl PropagationResult {
    #[doc = include_str!("propagation_result.doc/new.md")]
    #[must_use]
    pub const fn new(geometry_changed: bool, contained: bool) -> Self {
        Self {
            geometry_changed,
            contained,
        }
    }

    #[doc = include_str!("propagation_result.doc/from_pass.md")]
    #[must_use]
    pub const fn from_pass(pass: &LayoutPassResult, contained: bool) -> Self {
        Self::new(pass.geometry_changed(), contained)
    }

    #[doc = include_str!("propagation_result.doc/geometry_changed_2.md")]
    #[must_use]
    pub const fn geometry_changed(&self) -> bool {
        self.geometry_changed
    }

    #[doc = include_str!("propagation_result.doc/contained_2.md")]
    #[must_use]
    pub const fn contained(&self) -> bool {
        self.contained
    }

    #[doc = include_str!("propagation_result.doc/continues_upward.md")]
    #[must_use]
    pub const fn continues_upward(&self) -> bool {
        self.geometry_changed && !self.contained
    }
}

#[cfg(test)]
mod tests {
    use super::PropagationResult;
    use crate::model::LayoutBox;
    use crate::pass::LayoutPassResult;
    use obscur_geometry::Size;

    fn a_box() -> LayoutBox {
        LayoutBox::from_size(Size::new(10.0, 10.0))
    }

    #[test]
    fn continues_only_when_changed_and_not_contained() {
        assert!(PropagationResult::new(true, false).continues_upward());
        assert!(!PropagationResult::new(false, false).continues_upward());
        assert!(!PropagationResult::new(true, true).continues_upward());
        assert!(!PropagationResult::new(false, true).continues_upward());
    }

    #[test]
    fn from_pass_reads_geometry_flag() {
        let changed = LayoutPassResult::changed(a_box());
        let unchanged = LayoutPassResult::unchanged(a_box());

        let r = PropagationResult::from_pass(&changed, false);
        assert!(r.geometry_changed());
        assert!(r.continues_upward());

        let r = PropagationResult::from_pass(&unchanged, false);
        assert!(!r.geometry_changed());
        assert!(!r.continues_upward());
    }

    #[test]
    fn from_pass_respects_containment() {
        let changed = LayoutPassResult::changed(a_box());
        let r = PropagationResult::from_pass(&changed, true);
        assert!(r.geometry_changed());
        assert!(r.contained());
        assert!(!r.continues_upward());
    }
}
