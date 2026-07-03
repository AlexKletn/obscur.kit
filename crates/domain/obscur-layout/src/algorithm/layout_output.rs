//! `LayoutOutput`: the result of laying out one node.

use obscur_geometry::Size;

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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayoutOutput {
    size: Size,
    child_boxes: Vec<LayoutBox>,
}

impl LayoutOutput {
    /// Creates an output from a node's border-box `size` and its arranged
    /// `child_boxes` (in this node's coordinate space, parallel to the children).
    #[must_use]
    pub fn new(size: Size, child_boxes: Vec<LayoutBox>) -> Self {
        Self { size, child_boxes }
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
    fn empty_when_no_children() {
        let out = LayoutOutput::new(Size::new(5.0, 5.0), Vec::new());
        assert!(out.is_empty());
        assert_eq!(out.child_boxes(), &[]);
    }
}
