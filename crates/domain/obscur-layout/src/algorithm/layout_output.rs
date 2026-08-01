#![doc = include_str!("layout_output.doc/_module.md")]

use obscur_geometry::Size;

use super::ChildPercentBasis;
use crate::model::LayoutBox;

#[doc = include_str!("layout_output.doc/LayoutOutput.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayoutOutput {
    size: Size,
    child_boxes: Vec<LayoutBox>,
    #[doc = include_str!("layout_output.doc/child_percent_bases.md")]
    child_percent_bases: Vec<ChildPercentBasis>,
}

impl LayoutOutput {
    #[doc = include_str!("layout_output.doc/new.md")]
    #[must_use]
    pub fn new(size: Size, child_boxes: Vec<LayoutBox>) -> Self {
        Self {
            size,
            child_boxes,
            child_percent_bases: Vec::new(),
        }
    }

    #[doc = include_str!("layout_output.doc/with_child_percent_bases.md")]
    #[must_use]
    pub fn with_child_percent_bases(mut self, bases: Vec<ChildPercentBasis>) -> Self {
        self.child_percent_bases = bases;
        self
    }

    #[doc = include_str!("layout_output.doc/child_percent_basis.md")]
    #[must_use]
    pub fn child_percent_basis(&self, index: usize) -> ChildPercentBasis {
        self.child_percent_bases
            .get(index)
            .copied()
            .unwrap_or(ChildPercentBasis::UNSTATED)
    }

    #[doc = include_str!("layout_output.doc/size.md")]
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    #[doc = include_str!("layout_output.doc/child_boxes.md")]
    #[must_use]
    pub fn child_boxes(&self) -> &[LayoutBox] {
        &self.child_boxes
    }

    #[doc = include_str!("layout_output.doc/is_empty.md")]
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
