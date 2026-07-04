//! `CustomLayout`: the labeled extension-point formatting context.

use obscur_geometry::{BoxConstraints, Point, Rect, Size};

use crate::model::{LayoutBox, LayoutStyle};

use super::{ChildMeasurer, LayoutAlgorithm, LayoutChild, LayoutOutput};

/// The custom / extension-point layout (spec §4.1, §4.2).
///
/// A real custom formatting context is user code: any type implementing
/// [`LayoutAlgorithm`] *is* a custom layout. `CustomLayout` is not that
/// mechanism — it is a **labeled fallback**: a named passthrough that lays its
/// children out in a single vertical stack, serving as a sane default and a
/// reference implementation for the extension point.
///
/// # Name
///
/// The `name` is a diagnostic identifier for the custom context (e.g. shown in
/// layout traces); it does not affect the geometry.
///
/// # Algorithm (vertical-stack default)
///
/// Behaves like a minimal flow: each child is measured under the content width
/// (`constraints.max.width` minus horizontal `padding`) with its natural height,
/// stacked top-to-bottom starting at the content origin, and separated from the
/// previous child by `style.gap`. The node size is the content width plus
/// padding by the accumulated child height plus padding, clamped into
/// `constraints`.
///
/// # Production custom layouts
///
/// Production code implements [`LayoutAlgorithm`] directly for its own type
/// rather than configuring this one; `CustomLayout` exists so the extension
/// point has a concrete, labeled default with defined behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomLayout {
    /// Diagnostic identifier for this custom context.
    name: Box<str>,
}

impl CustomLayout {
    /// Creates a custom layout labeled `name`.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self { name: name.into() }
    }

    /// The diagnostic name of this custom context.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for CustomLayout {
    /// A custom layout labeled `"custom"`.
    fn default() -> Self {
        Self::new("custom")
    }
}

impl LayoutAlgorithm for CustomLayout {
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
        let avail_height = (constraints.max.height - pad_y).max(0.0);

        let mut child_boxes = Vec::with_capacity(children.len());
        let mut y = 0.0_f32;
        for index in 0..children.len() {
            if index > 0 {
                y += style.gap;
            }
            let remaining_height = (avail_height - y).max(0.0);
            let size = measurer.measure_child(
                index,
                BoxConstraints::loose(Size::new(content_width, remaining_height)),
            );
            let origin = Point::new(padding.left, padding.top + y);
            child_boxes.push(LayoutBox::new(Rect::new(origin, size)));
            y += size.height;
        }

        let border_box = Size::new(content_width + pad_x, y + pad_y);
        let size = constraints.constrain(border_box);

        LayoutOutput::new(size, child_boxes)
    }
}

#[cfg(test)]
mod tests {
    use super::CustomLayout;
    use crate::algorithm::{FixedChildMeasurer, LayoutAlgorithm, LayoutChild};
    use crate::model::LayoutStyle;
    use obscur_geometry::{BoxConstraints, Insets, Point, Size};
    use obscur_identity::NodeId;

    fn child(raw: u64) -> LayoutChild {
        LayoutChild::new(NodeId::from_raw(raw), LayoutStyle::default())
    }

    #[test]
    fn default_is_named_custom() {
        assert_eq!(CustomLayout::default().name(), "custom");
        assert_eq!(CustomLayout::new("overlay").name(), "overlay");
    }

    #[test]
    fn stacks_children_vertically_with_gap() {
        let style = LayoutStyle {
            gap: 8.0,
            ..LayoutStyle::default()
        };
        let children = [child(1), child(2)];
        let mut measurer =
            FixedChildMeasurer::new(vec![Size::new(100.0, 20.0), Size::new(100.0, 30.0)]);
        let out = CustomLayout::new("stack").layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        let b = out.child_boxes();
        assert_eq!(b[0].origin(), Point::new(0.0, 0.0));
        assert_eq!(b[0].size(), Size::new(100.0, 20.0));
        // Second child below the first plus the gap: 20 + 8 = 28.
        assert_eq!(b[1].origin(), Point::new(0.0, 28.0));
        assert_eq!(b[1].size(), Size::new(100.0, 30.0));
        // Node height: 20 + 8 gap + 30 = 58.
        assert_eq!(out.size(), Size::new(100.0, 58.0));
    }

    #[test]
    fn padding_offsets_stack_and_grows_box() {
        let style = LayoutStyle {
            padding: Insets::uniform(5.0),
            ..LayoutStyle::default()
        };
        let children = [child(1)];
        let mut measurer = FixedChildMeasurer::new(vec![Size::new(90.0, 40.0)]);
        let out = CustomLayout::default().layout(
            &style,
            &children,
            BoxConstraints::loose(Size::new(100.0, 1000.0)),
            &mut measurer,
        );
        assert_eq!(out.child_boxes()[0].origin(), Point::new(5.0, 5.0));
        // Content width 90 + padding 10 = 100; height 40 + padding 10 = 50.
        assert_eq!(out.size(), Size::new(100.0, 50.0));
    }

    #[test]
    fn empty_container_sizes_to_content_width_and_padding() {
        let style = LayoutStyle {
            padding: Insets::uniform(3.0),
            ..LayoutStyle::default()
        };
        let mut measurer = FixedChildMeasurer::default();
        let out = CustomLayout::default().layout(
            &style,
            &[],
            BoxConstraints::loose(Size::new(50.0, 50.0)),
            &mut measurer,
        );
        assert!(out.is_empty());
        // Content width = 50 - 6 = 44; border box = 50 wide, 6 tall.
        assert_eq!(out.size(), Size::new(50.0, 6.0));
    }
}
