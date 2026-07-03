//! `LayoutAlgorithm`: the trait every formatting context implements.

use obscur_geometry::BoxConstraints;

use crate::model::LayoutStyle;

use super::{ChildMeasurer, LayoutChild, LayoutOutput};

/// The contract every formatting context (flow, flex, ...) implements to lay out
/// a single node.
///
/// # Measure + arrange in one pass
///
/// A `LayoutAlgorithm` performs the §4.2 measure and arrange phases for one node
/// together and returns both results in a [`LayoutOutput`]: the *measure* result
/// is the returned [`size`](LayoutOutput::size) (the node's border-box size under
/// `constraints`), and the *arrange* result is the returned
/// [`child_boxes`](LayoutOutput::child_boxes) (each child's box in this node's
/// coordinate space). Framing it as two phases still holds — the algorithm first
/// determines child and self sizes, then positions the children — but the runtime
/// consumes them from a single call.
///
/// # Decoupled from the tree
///
/// An algorithm never walks the retained tree. It is handed the node's own
/// `style`, the slice of its `children` (identity + style only — see
/// [`LayoutChild`]), the `constraints` under which it must size itself, and a
/// [`ChildMeasurer`] it calls to size any child under constraints of its
/// choosing. The runtime owns traversal and, behind the measurer, decides
/// whether a child is a leaf (routed to the intrinsic-measure port) or a nested
/// container (laid out recursively).
///
/// # Dispatch
///
/// The runtime selects the algorithm from `style.display`:
/// [`Block`](crate::model::LayoutDisplay::Block) uses
/// [`FlowLayout`](super::FlowLayout),
/// [`Flex`](crate::model::LayoutDisplay::Flex) uses
/// [`FlexLayout`](super::FlexLayout), and
/// [`None`](crate::model::LayoutDisplay::None) generates no box and is not laid
/// out at all. Phase 1 ships only flow and flex; grid, positioned, docking, and
/// custom contexts are deferred.
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
