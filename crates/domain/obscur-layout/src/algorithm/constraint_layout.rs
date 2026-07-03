//! ConstraintLayout: a minimal constraint / overlay formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{LayoutBox, LayoutStyle};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// Minimal constraint / overlay layout (spec §4.1, §4.2).
///
/// Every child is constrained to the container's content box and placed at the
/// content origin, so all children **overlay** one another at the top-left. This
/// is the simplest possible constraint context: children share the container's
/// bounds rather than being related to one another.
///
/// # Algorithm
///
/// 1. Each child is measured under the container's content area (its
///    `constraints.max` minus `padding`, clamped to be non-negative), loosened
///    so a child may be smaller than the container.
/// 2. Each child is placed at `(padding.left, padding.top)`.
/// 3. The node size is the container's own `width` / `height` when definite
///    (resolved against `constraints.max`), otherwise the largest child size
///    plus the container `padding`. The result is clamped into `constraints`.
///
/// # Minimal constraint context
///
/// This is deliberately minimal: there are **no relative inter-child
/// constraints** (align/anchor edges to sibling edges, chains, bias, guidelines,
/// barriers). Those belong to a fuller constraint solver and are future work;
/// here every child is simply pinned to the container origin and overlaid.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ConstraintLayout;

impl ConstraintLayout {
    /// Creates a minimal constraint / overlay layout.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl LayoutAlgorithm for ConstraintLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let padding = style.padding;
        let pad_x = padding.left + padding.right;
        let pad_y = padding.top + padding.bottom;

        let content_width = (constraints.max.width - pad_x).max(0.0);
        let content_height = (constraints.max.height - pad_y).max(0.0);
        let child_constraints = BoxConstraints::loose(Size::new(content_width, content_height));

        let origin = Point::new(padding.left, padding.top);
        let mut child_boxes = Vec::with_capacity(children.len());
        let mut max_child = Size::ZERO;

        for index in 0..children.len() {
            let size = measurer.measure_child(index, child_constraints);
            child_boxes.push(LayoutBox::new(Rect::new(origin, size)));
            max_child = Size::new(
                max_child.width.max(size.width),
                max_child.height.max(size.height),
            );
        }

        // Definite own size wins; otherwise the largest child plus padding.
        let width = style
            .width
            .resolve(constraints.max.width)
            .unwrap_or(max_child.width + pad_x);
        let height = style
            .height
            .resolve(constraints.max.height)
            .unwrap_or(max_child.height + pad_y);
        let size = constraints.constrain(Size::new(width.max(0.0), height.max(0.0)));

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::ConstraintLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{Dimension, LayoutStyle};
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
    }

    #[test]
    fn overlays_children_at_origin() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(40.0, 30.0), Size::new(60.0, 20.0)]);
        let out = ConstraintLayout::new().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 200.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        // Both children pinned to the content origin (overlay).
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[1].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size(), Size::new(40.0, 30.0));
        assert_eq!(b[1].size(), Size::new(60.0, 20.0));
        // Node sizes to the per-axis maximum child extent: 60 wide, 30 tall.
        assert_eq!(out.size(), Size::new(60.0, 30.0));
    }

    #[test]
    fn definite_size_overrides_max_child() {
        let style = LayoutStyle::default()
            .with_width(Dimension::Px(120.0))
            .with_height(Dimension::Px(80.0));
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(40.0, 30.0)]);
        let out = ConstraintLayout::new().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(300.0, 300.0)),
            &mut measurer,
        );
        assert_eq!(out.size(), Size::new(120.0, 80.0));
    }

    #[test]
    fn padding_offsets_children_and_grows_box() {
        let style = LayoutStyle {
            padding: Insets::uniform(5.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(40.0, 40.0)]);
        let out = ConstraintLayout::new().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 200.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(5.0, 5.0));
        // Max child 40 x 40 plus padding 10 on each axis.
        assert_eq!(out.size(), Size::new(50.0, 50.0));
    }

    #[test]
    fn empty_container_sizes_to_padding() {
        let style = LayoutStyle {
            padding: Insets::uniform(4.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::default();
        let out = ConstraintLayout::new().layout(
            &style,
            &[],
            BoxConstraints::loose(Size::new(50.0, 50.0)),
            &mut measurer,
        );
        assert!(out.is_empty());
        assert_eq!(out.size(), Size::new(8.0, 8.0));
    }
}
