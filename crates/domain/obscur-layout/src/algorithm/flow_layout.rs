//! `FlowLayout`: normal-flow (block) layout for the `Block` formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{LayoutBox, LayoutStyle};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// Normal-flow / block layout (spec §4.1, §4.2).
///
/// Stacks children top-to-bottom in document order, each occupying the full
/// content width, and sizes the node to fit unless an explicit height is given.
///
/// # Phase-1 rules
///
/// * **Content width** — resolves `style.width` against the constraint's max
///   width; if `Auto`, uses the available content width (`constraints.max.width`
///   minus horizontal border + padding). Clamped to be non-negative.
/// * **Child stacking** — each child is measured with
///   [`BoxConstraints::loose`] up to `(content_width, remaining_height)`, placed
///   at the content-box left edge, with its top at the running `y` offset.
///   Consecutive children are separated by `style.gap` (Phase 1 treats `gap` as
///   block spacing between children).
/// * **Content height** — resolves `style.height` if set, otherwise the summed
///   height of the children (including gaps). Clamped to be non-negative.
/// * **Border box** — content size plus border and padding, then clamped into
///   the incoming `constraints`.
///
/// Child boxes are positioned relative to this node's origin, already offset into
/// the content box by border + padding.
///
/// # Out of scope for Phase 1
///
/// Margin collapsing, floats, inline formatting / line boxes, horizontal
/// alignment, and writing-mode-aware block direction are not modelled; block
/// flow is strictly vertical with children pinned to the content-box left edge.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlowLayout;

impl LayoutAlgorithm for FlowLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let border = style.border;
        let padding = style.padding;
        let bp_left = border.left + padding.left;
        let bp_top = border.top + padding.top;
        let bp_x = bp_left + border.right + padding.right;
        let bp_y = bp_top + border.bottom + padding.bottom;

        // Content width: explicit width (resolved against the max width) else the
        // available content width. Never negative.
        let content_width = style
            .width
            .resolve(constraints.max.width)
            .unwrap_or(constraints.max.width - bp_x)
            .max(0.0);

        // Available content height for measuring children under (may be infinite).
        let avail_content_height = (constraints.max.height - bp_y).max(0.0);

        let mut child_boxes = Vec::with_capacity(children.len());
        let mut y = 0.0_f32;
        for (index, _child) in children.iter().enumerate() {
            if index > 0 {
                y += style.gap;
            }
            let remaining_height = (avail_content_height - y).max(0.0);
            let child_size = measurer.measure_child(
                index,
                BoxConstraints::loose(Size::new(content_width, remaining_height)),
            );
            let origin = Point::new(bp_left, bp_top + y);
            child_boxes.push(LayoutBox::new(Rect::new(origin, child_size)));
            y += child_size.height;
        }

        // Content height: explicit height else the accumulated children height.
        let content_height = style
            .height
            .resolve(constraints.max.height)
            .unwrap_or(y)
            .max(0.0);

        let border_box = Size::new(content_width + bp_x, content_height + bp_y);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::FlowLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{Dimension, LayoutStyle};
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
    }

    #[test]
    fn stacks_two_children_and_sizes_to_fit() {
        let style = LayoutStyle::default();
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(100.0, 20.0), Size::new(100.0, 30.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );

        let boxes = out.child_boxes();
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[0].size(), Size::new(100.0, 20.0));
        // Second child sits directly below the first (no gap).
        assert_eq!(boxes[1].origin(), Point::new(0.0, 20.0));
        assert_eq!(boxes[1].size(), Size::new(100.0, 30.0));
        // Node sizes to fit the stacked children.
        assert_eq!(out.size(), Size::new(100.0, 50.0));
    }

    #[test]
    fn gap_separates_children() {
        let style = LayoutStyle {
            gap: 8.0,
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(100.0, 20.0), Size::new(100.0, 30.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );

        let boxes = out.child_boxes();
        assert_eq!(boxes[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[1].origin(), Point::new(0.0, 28.0));
        // 20 + 8 gap + 30 = 58.
        assert_eq!(out.size(), Size::new(100.0, 58.0));
    }

    #[test]
    fn fixed_height_overrides_content_height() {
        let style = LayoutStyle::default().with_height(Dimension::Px(200.0));
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(100.0, 20.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.size(), Size::new(100.0, 200.0));
    }

    #[test]
    fn border_and_padding_offset_children_and_grow_box() {
        let style = LayoutStyle {
            padding: Insets::uniform(4.0),
            border: Insets::uniform(1.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 10.0)]);
        let out = FlowLayout.layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        // Content origin offset by border(1) + padding(4) = 5 on each axis.
        assert_eq!(out.child_boxes()[0].origin(), Point::new(5.0, 5.0));
        // Content width = 100 - 10 = 90; border box = 90 + 10 = 100 wide.
        // Content height = 10; border box = 10 + 10 = 20 tall.
        assert_eq!(out.size(), Size::new(100.0, 20.0));
    }

    #[test]
    fn empty_children_size_to_insets_only() {
        let style = LayoutStyle {
            padding: Insets::uniform(3.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::default();
        let out = FlowLayout.layout(
            &style,
            &[],
            BoxConstraints::loose(Size::new(50.0, 50.0)),
            &mut measurer,
        );
        assert!(out.is_empty());
        // Content width = 50 - 6 = 44, height = 0; border box = 50 x 6.
        assert_eq!(out.size(), Size::new(50.0, 6.0));
    }
}
