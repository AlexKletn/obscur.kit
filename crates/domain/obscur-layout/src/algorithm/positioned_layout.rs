//! PositionedLayout: absolute positioning of children within a container.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{LayoutBox, LayoutStyle};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// Absolute-positioning layout (spec §4.1, §4.2).
///
/// Every child is measured at its natural size and placed at its own offset
/// within the container's content box, independent of its siblings — children
/// may overlap freely. This is the "positioned" / absolute formatting context.
///
/// # Offsets come from each child's margin
///
/// The model has no dedicated inset (`top`/`left`) properties yet, so the offset
/// of each child is taken from its **margin**: a child is placed at
/// `(padding.left + child.margin.left, padding.top + child.margin.top)`. This is
/// a pragmatic stand-in until dedicated positioning-offset properties exist.
///
/// # Algorithm
///
/// 1. Each child is measured under `constraints` **loosened** to its container
///    max, so it reports its natural size rather than being stretched.
/// 2. Each child is placed at its margin offset inside the content box.
/// 3. The node size is the container's own `width` / `height` when definite
///    (resolved against `constraints.max`), otherwise the bounding box of the
///    positioned children plus the container `padding`. The result is clamped
///    into `constraints`.
///
/// # Out of scope
///
/// Containing-block resolution, right/bottom anchoring, `z-index` stacking
/// order, and static-position fallback are not modelled; placement is strictly
/// top-left offset by the child's margin.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PositionedLayout;

impl PositionedLayout {
    /// Creates an absolute-positioning layout.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl LayoutAlgorithm for PositionedLayout {
    fn layout(
        &self,
        style: &LayoutStyle,
        children: &[LayoutChild],
        constraints: BoxConstraints,
        measurer: &mut dyn ChildMeasurer,
    ) -> LayoutOutput {
        let padding = style.padding;

        // Children measure at their natural size: loosen so nothing is forced.
        let child_constraints = BoxConstraints::loose(constraints.max);

        let mut child_boxes = Vec::with_capacity(children.len());
        // Content-relative extents; start at the padding origin so an empty
        // container sizes to its padding only.
        let mut max_right = padding.left;
        let mut max_bottom = padding.top;

        for (index, child) in children.iter().enumerate() {
            let margin = child.style.margin;
            let size = measurer.measure_child(index, child_constraints);
            let origin = Point::new(padding.left + margin.left, padding.top + margin.top);
            child_boxes.push(LayoutBox::new(Rect::new(origin, size)));
            max_right = max_right.max(origin.x + size.width);
            max_bottom = max_bottom.max(origin.y + size.height);
        }

        // Definite own size wins; otherwise the positioned bounding box.
        let width = style
            .width
            .resolve(constraints.max.width)
            .unwrap_or(max_right + padding.right);
        let height = style
            .height
            .resolve(constraints.max.height)
            .unwrap_or(max_bottom + padding.bottom);
        let size = constraints.constrain(Size::new(width.max(0.0), height.max(0.0)));

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::PositionedLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::{Dimension, LayoutStyle};
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child_at(raw: u64, left: f32, top: f32) -> LayoutChild {
        let style = LayoutStyle {
            margin: Insets {
                top,
                left,
                ..Insets::default()
            },
            ..LayoutStyle::default()
        };
        LayoutChild::new(NodeId::from_raw(raw), style)
    }

    #[test]
    fn places_children_at_margin_offsets() {
        let style = LayoutStyle::default();
        let children = [child_at(1, 10.0, 20.0), child_at(2, 30.0, 5.0)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(40.0, 40.0), Size::new(50.0, 60.0)]);
        let out = PositionedLayout::new().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 200.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(10.0, 20.0));
        assert_eq!(b[0].size(), Size::new(40.0, 40.0));
        assert_eq!(b[1].origin(), Point::new(30.0, 5.0));
        assert_eq!(b[1].size(), Size::new(50.0, 60.0));
        // Bounding box: child1 right = 30 + 50 = 80; child0 bottom = 20 + 40 = 60,
        // child1 bottom = 5 + 60 = 65 (the taller extent).
        assert_eq!(out.size(), Size::new(80.0, 65.0));
    }

    #[test]
    fn definite_size_overrides_bounding_box() {
        let style = LayoutStyle::default()
            .with_width(Dimension::Px(300.0))
            .with_height(Dimension::Px(150.0));
        let children = [child_at(1, 0.0, 0.0)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(40.0, 40.0)]);
        let out = PositionedLayout::new().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(500.0, 500.0)),
            &mut measurer,
        );
        assert_eq!(out.size(), Size::new(300.0, 150.0));
    }

    #[test]
    fn padding_offsets_children() {
        let style = LayoutStyle {
            padding: Insets::uniform(6.0),
            ..LayoutStyle::default()
        };
        let children = [child_at(1, 4.0, 4.0)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(10.0, 10.0)]);
        let out = PositionedLayout::new().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(200.0, 200.0)),
            &mut measurer,
        );
        // Origin = padding(6) + margin(4) = 10 on each axis.
        assert_eq!(out.child_boxes()[0].origin(), Point::new(10.0, 10.0));
    }

    #[test]
    fn empty_container_sizes_to_padding() {
        let style = LayoutStyle {
            padding: Insets::uniform(3.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::default();
        let out = PositionedLayout::new().layout(
            &style,
            &[],
            BoxConstraints::loose(Size::new(50.0, 50.0)),
            &mut measurer,
        );
        assert!(out.is_empty());
        assert_eq!(out.size(), Size::new(6.0, 6.0));
    }
}
