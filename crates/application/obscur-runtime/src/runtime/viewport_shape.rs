use obscur_geometry::{CornerRadii, CornerRadius, Point, Rect, Size};
use obscur_scene::clip::Clip;
use obscur_tree::tree::RetainedTree;

use crate::orchestration::{ComputedStyles, LayoutBoxes, corner_radii_of};

pub(super) fn radii(
    tree: &RetainedTree,
    boxes: &LayoutBoxes,
    computed: &ComputedStyles,
    viewport: Size,
) -> CornerRadii {
    let Some(document) = tree.root().and_then(|root| tree.get(root)) else {
        return CornerRadii::default();
    };
    let &[root] = document.child_nodes() else {
        return CornerRadii::default();
    };
    let Some(layout) = boxes.get(&root) else {
        return CornerRadii::default();
    };
    let bounds = layout.bounds;
    if bounds.origin.x.abs() > 0.01
        || bounds.origin.y.abs() > 0.01
        || (bounds.size.width - viewport.width).abs() > 0.01
        || (bounds.size.height - viewport.height).abs() > 0.01
    {
        return CornerRadii::default();
    }
    corner_radii_of(computed.get(&root), Some(bounds)).normalized(bounds)
}

/// The window mask stays in surface coordinates when content pans or zooms.
pub(super) fn clip(viewport: Size, device_scale: f32, radii: CornerRadii) -> Option<Clip> {
    if !radii.corners().into_iter().any(CornerRadius::is_rounded) {
        return None;
    }
    let scale =
        |corner: CornerRadius| CornerRadius::new(corner.x * device_scale, corner.y * device_scale);
    Some(Clip::rounded_rect(
        Rect::new(
            Point::ORIGIN,
            Size::new(
                viewport.width * device_scale,
                viewport.height * device_scale,
            ),
        ),
        CornerRadii {
            top_left: scale(radii.top_left),
            top_right: scale(radii.top_right),
            bottom_right: scale(radii.bottom_right),
            bottom_left: scale(radii.bottom_left),
        },
    ))
}
