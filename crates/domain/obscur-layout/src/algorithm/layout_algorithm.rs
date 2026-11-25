//! `LayoutAlgorithm`: the trait every formatting context implements.

use obscur_geometry::BoxConstraints;

use crate::model::LayoutStyle;

use super::{ChildMeasurer, LayoutChild, LayoutOutput};

#[doc = include_str!("../../docs/api/algorithm/layout_algorithm.doc/LayoutAlgorithm.md")]
pub trait LayoutAlgorithm {
    /// Lays out `style`'s node with the given `children` under `constraints`,
    /// measuring children through `measurer`, and returns the node's border-box
    /// size together with each child's arranged box.
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
