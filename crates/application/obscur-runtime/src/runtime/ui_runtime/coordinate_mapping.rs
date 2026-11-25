//! Coordinate mappings derived from committed runtime geometry.

use obscur_geometry::space::{
    LocalSpace, NormalizedSpace, ParentSpace, PhysicalSpace, PointIn, RectIn, ScreenSpace,
    WindowSpace, WorldSpace,
};
use obscur_geometry::{AffineTransform, CoordinateMapper, PixelSnapMode, PixelSnapPolicy, Rect};
use obscur_identity::NodeId;

use crate::orchestration::{
    ComputedStyles, ScrollContainer, about_transform_origin, resolved_transform, scroll_containers,
};

use super::UiRuntime;

impl UiRuntime {
    /// Applies an explicit pixel snap mode to a paint rectangle in logical window
    /// coordinates using the current device scale. Layout and hit testing retain
    /// their original geometry.
    #[must_use]
    pub fn snap_window_paint_rect(&self, rect: Rect, mode: PixelSnapMode) -> Rect {
        PixelSnapPolicy::new(mode).snap_rect(rect, self.device_scale())
    }
    /// Maps node-local coordinates into its visual parent's space relative to the
    /// parent's layout-box origin. Includes the node's CSS transform and the
    /// parent's scroll offset. Requires committed boxes for both nodes.
    #[must_use]
    pub fn local_to_parent(
        &self,
        node: NodeId,
    ) -> Option<CoordinateMapper<LocalSpace, ParentSpace>> {
        let parent = self.tree.visual_parent_of(node)?;
        let parent_origin = self.boxes.get(&parent)?.origin();
        let layout = self.boxes.get(&node)?;
        let origin = layout.origin();
        let own = resolved_transform(self.computed.get(&node), layout.bounds)
            .unwrap_or(AffineTransform::IDENTITY);
        let scroll = scroll_containers(&self.tree, &self.computed, &self.boxes)
            .into_iter()
            .find(|container| container.node == parent)
            .map_or_else(Default::default, |container| self.scroll.offset(container));
        let to_parent =
            AffineTransform::translate(-parent_origin.x - scroll.x, -parent_origin.y - scroll.y);
        Some(CoordinateMapper::new(
            to_parent
                .multiply(own)
                .multiply(AffineTransform::translate(origin.x, origin.y)),
        ))
    }

    /// Maps the node's local layout box to fractions of its width and height.
    /// Points outside the box can produce values outside the unit interval.
    /// Returns none when the box is missing or either axis cannot be normalized.
    #[must_use]
    pub fn local_to_normalized(
        &self,
        node: NodeId,
    ) -> Option<CoordinateMapper<LocalSpace, NormalizedSpace>> {
        let size = self.boxes.get(&node)?.size();
        let (x, y) = (size.width.recip(), size.height.recip());
        (x.is_finite() && y.is_finite() && x > 0.0 && y > 0.0)
            .then(|| CoordinateMapper::new(AffineTransform::scale(x, y)))
    }

    /// Maps points relative to the node's layout-box origin into window logical
    /// coordinates. Includes CSS transforms, visual ancestors, ancestor scrolling
    /// and the surface content offset. Returns none without committed layout.
    #[must_use]
    pub fn local_to_window(
        &self,
        node: NodeId,
    ) -> Option<CoordinateMapper<LocalSpace, WindowSpace>> {
        let scrolls = scroll_containers(&self.tree, &self.computed, &self.boxes);
        self.local_to_window_affine(node, &scrolls)
            .map(CoordinateMapper::new)
    }

    /// Maps node-local coordinates to the window using the supplied scroll containers.
    pub(super) fn local_to_window_affine(
        &self,
        node: NodeId,
        scrolls: &[ScrollContainer],
    ) -> Option<AffineTransform> {
        self.local_to_window_with_styles(node, scrolls, &self.computed)
    }

    /// Builds a mapping using a frame’s resolved styles and scroll geometry.
    pub(super) fn local_to_window_with_styles(
        &self,
        node: NodeId,
        scrolls: &[ScrollContainer],
        computed: &ComputedStyles,
    ) -> Option<AffineTransform> {
        self.map_local_to_window(node, scrolls, computed, |_, _, _| None)
    }

    /// Maps the last presented node and visual ancestor transforms to the logical window.
    pub(super) fn presented_local_to_window(
        &self,
        node: NodeId,
        scrolls: &[ScrollContainer],
    ) -> Option<AffineTransform> {
        self.map_local_to_window(node, scrolls, &self.computed, |node, styles, bounds| {
            let matrix = self.motion.presented_transform(node, bounds.size)?;
            Some(about_transform_origin(styles.get(&node), bounds, matrix))
        })
    }

    fn map_local_to_window(
        &self,
        node: NodeId,
        scrolls: &[ScrollContainer],
        computed: &ComputedStyles,
        presented: impl Fn(NodeId, &ComputedStyles, Rect) -> Option<AffineTransform>,
    ) -> Option<AffineTransform> {
        let origin = self.boxes.get(&node)?.origin();
        let mut transform = AffineTransform::translate(origin.x, origin.y);
        let mut current = node;
        for _ in 0..self.tree.len() {
            if let Some(own) = self.boxes.get(&current).and_then(|layout| {
                presented(current, computed, layout.bounds)
                    .or_else(|| resolved_transform(computed.get(&current), layout.bounds))
            }) {
                transform = own.multiply(transform);
            }
            let Some(parent) = self.tree.visual_parent_of(current) else {
                return Some(self.surface.window_transform().multiply(transform));
            };
            if let Some(container) = scrolls.iter().find(|entry| entry.node == parent) {
                let offset = self.scroll.offset(*container);
                transform = AffineTransform::translate(-offset.x, -offset.y).multiply(transform);
            }
            current = parent;
        }
        None
    }

    /// Maps scene coordinates through the current content offset and camera into logical window coordinates.
    #[must_use]
    pub fn world_to_window(&self) -> CoordinateMapper<WorldSpace, WindowSpace> {
        CoordinateMapper::new(self.surface.window_transform())
    }

    /// Returns the visible window region tagged as world coordinates for spatial queries and selection.
    #[must_use]
    pub fn visible_world_rect(&self) -> RectIn<WorldSpace> {
        RectIn::new(self.visible_content_rect())
    }

    /// Maps window logical coordinates to physical pixels using the current device scale.
    #[must_use]
    pub fn window_to_physical(&self) -> CoordinateMapper<WindowSpace, PhysicalSpace> {
        let scale = self.device_scale();
        CoordinateMapper::new(AffineTransform::scale(scale, scale))
    }

    /// Returns the presented border box in physical pixels. The result is an
    /// axis-aligned rectangle after transforms, scrolling and device scale.
    #[must_use]
    pub fn presented_physical_bounds(&self, node: NodeId) -> Option<Rect> {
        if self.is_css_hidden(node) {
            return None;
        }
        let scrolls = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let size = self.boxes.get(&node)?.size();
        let scale = self.device_scale();
        let transform = AffineTransform::scale(scale, scale)
            .multiply(self.presented_local_to_window(node, &scrolls)?);
        let bounds = transform.map_rect(Rect::new(obscur_geometry::Point::ORIGIN, size));
        [
            bounds.origin.x,
            bounds.origin.y,
            bounds.right(),
            bounds.bottom(),
        ]
        .iter()
        .all(|value| value.is_finite())
        .then_some(bounds)
    }

    /// Maps logical window coordinates to desktop physical pixels using the current
    /// device scale and the host-supplied origin of the window's client area.
    #[must_use]
    pub fn window_to_screen(
        &self,
        origin: PointIn<ScreenSpace>,
    ) -> CoordinateMapper<WindowSpace, ScreenSpace> {
        let origin = origin.point();
        self.window_to_physical()
            .then(&CoordinateMapper::new(AffineTransform::translate(
                origin.x, origin.y,
            )))
    }
}
