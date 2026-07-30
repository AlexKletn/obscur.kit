//! `LayoutOutput`: the result of laying out one node.

use obscur_geometry::Size;

use super::ChildPercentBasis;
use crate::model::LayoutBox;

/// The result a [`LayoutAlgorithm`] produces for a single node: the node's own
/// border-box size, plus the arranged box of each child.
///
/// [`LayoutAlgorithm`]: super::LayoutAlgorithm
///
/// This bundles the two outputs of the §4.2 measure/arrange contract for one
/// node: `size` is the node's resolved **border-box** size (the *measure*
/// result), and `child_boxes[i]` is child `i`'s [`LayoutBox`] (the *arrange*
/// result). Child boxes are expressed **in this node's coordinate space**: each
/// origin is relative to this node's origin and is already offset into the
/// node's content box (past its border and padding). The runtime translates them
/// into their parent's space as it walks back up the tree.
///
/// `child_boxes` is parallel to the `children` slice the algorithm was given:
/// `child_boxes[i]` positions `children[i]`.
///
/// # The third output: what a child's percentages resolve against
///
/// A child's box says where the child *is*; it does not say what containing
/// block the child's own `width: 50%` resolves against. For block flow and flex
/// those coincide with the parent's content box and the runtime supplies it
/// unaided, but a grid item's containing block is its grid area
/// (`specs/w3c/css-grid-2.txt:712`) — a fact only the grid algorithm holds.
/// [`child_percent_basis`](Self::child_percent_basis) is that fact, per child and
/// per axis; it defaults to [`ChildPercentBasis::UNSTATED`], so every algorithm
/// that does not set it behaves exactly as it did.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayoutOutput {
    size: Size,
    child_boxes: Vec<LayoutBox>,
    /// Parallel to `child_boxes` when stated, and **empty** when the algorithm
    /// stated nothing (the overwhelmingly common case, and the reason this is not
    /// a `Vec<Option<…>>` sized to match): an out-of-range index reads as
    /// [`ChildPercentBasis::UNSTATED`].
    child_percent_bases: Vec<ChildPercentBasis>,
}

impl LayoutOutput {
    /// Creates an output from a node's border-box `size` and its arranged
    /// `child_boxes` (in this node's coordinate space, parallel to the children).
    #[must_use]
    pub fn new(size: Size, child_boxes: Vec<LayoutBox>) -> Self {
        Self {
            size,
            child_boxes,
            child_percent_bases: Vec::new(),
        }
    }

    /// Attaches the containing block each child's percentages resolve against,
    /// parallel to the child boxes.
    ///
    /// Only an algorithm whose children have a containing block *other* than its
    /// own content box needs this — in practice grid, whose items are contained
    /// by their grid areas. See the type-level note on [`LayoutOutput`].
    #[must_use]
    pub fn with_child_percent_bases(mut self, bases: Vec<ChildPercentBasis>) -> Self {
        self.child_percent_bases = bases;
        self
    }

    /// What child `index`'s percentage sizes resolve against, per axis.
    ///
    /// [`ChildPercentBasis::UNSTATED`] when the algorithm said nothing about this
    /// child — the caller then uses the basis it would have used anyway.
    #[must_use]
    pub fn child_percent_basis(&self, index: usize) -> ChildPercentBasis {
        self.child_percent_bases
            .get(index)
            .copied()
            .unwrap_or(ChildPercentBasis::UNSTATED)
    }

    /// The node's resolved border-box size.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// The arranged child boxes, in this node's coordinate space.
    #[must_use]
    pub fn child_boxes(&self) -> &[LayoutBox] {
        &self.child_boxes
    }

    /// Returns `true` if the node has no arranged children.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.child_boxes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutOutput;
    use crate::model::LayoutBox;
    use obscur_geometry::{Point, Rect, Size};

    #[test]
    fn accessors_read_fields() {
        let boxes = vec![LayoutBox::new(Rect::new(
            Point::new(1.0, 2.0),
            Size::new(3.0, 4.0),
        ))];
        let out = LayoutOutput::new(Size::new(10.0, 20.0), boxes.clone());
        assert_eq!(out.size(), Size::new(10.0, 20.0));
        assert_eq!(out.child_boxes(), boxes.as_slice());
        assert!(!out.is_empty());
    }

    #[test]
    fn a_percentage_basis_is_unstated_unless_the_algorithm_states_it() {
        use super::super::ChildPercentBasis;
        let boxes = vec![LayoutBox::new(Rect::new(
            Point::new(0.0, 0.0),
            Size::new(3.0, 4.0),
        ))];
        let bare = LayoutOutput::new(Size::new(10.0, 20.0), boxes.clone());
        assert_eq!(bare.child_percent_basis(0), ChildPercentBasis::UNSTATED);
        let stated = LayoutOutput::new(Size::new(10.0, 20.0), boxes)
            .with_child_percent_bases(vec![ChildPercentBasis::with_width(211.0)]);
        assert_eq!(
            stated.child_percent_basis(0),
            ChildPercentBasis::with_width(211.0)
        );
        // A child past the stated end reads as unstated rather than panicking.
        assert_eq!(stated.child_percent_basis(9), ChildPercentBasis::UNSTATED);
    }

    #[test]
    fn empty_when_no_children() {
        let out = LayoutOutput::new(Size::new(5.0, 5.0), Vec::new());
        assert!(out.is_empty());
        assert_eq!(out.child_boxes(), &[]);
    }
}
