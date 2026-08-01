#![doc = include_str!("layout_algorithm.doc/_module.md")]

use obscur_geometry::BoxConstraints;

use crate::model::LayoutStyle;

use super::{ChildMeasurer, LayoutChild, LayoutOutput};

#[doc = include_str!("layout_algorithm.doc/LayoutAlgorithm.md")]
pub trait LayoutAlgorithm {
    #[doc = include_str!("layout_algorithm.doc/layout.md")]
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput;
}

#[cfg(test)]
mod tests {
    use super::LayoutAlgorithm;
    use crate::algorithm::{ChildMeasurer, LayoutChild, LayoutOutput};
    use crate::model::LayoutStyle;
    use obscur_geometry::{BoxConstraints, Size};

    /// A trivial context confirming the trait is object-safe and its arguments
    /// line up with the supporting types.
    struct Empty;

    impl LayoutAlgorithm for Empty {
        fn layout(
            &self,
            _style: &LayoutStyle,
            _children: &[LayoutChild],
            constraints: BoxConstraints,
            _measurer: &mut dyn ChildMeasurer,
        ) -> LayoutOutput {
            LayoutOutput::new(constraints.constrain(Size::ZERO), Vec::new())
        }
    }

    #[test]
    fn usable_behind_dyn() {
        let algo: &dyn LayoutAlgorithm = &Empty;
        let mut measurer = crate::algorithm::FixedChildMeasurer::default();
        let out = algo.layout(
            &LayoutStyle::default(),
            &[],
            BoxConstraints::tight(Size::new(10.0, 10.0)),
            &mut measurer,
        );
        assert_eq!(out.size(), Size::new(10.0, 10.0));
        assert!(out.is_empty());
    }
}
