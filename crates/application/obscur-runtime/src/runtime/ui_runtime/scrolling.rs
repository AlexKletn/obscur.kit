#![doc = include_str!("scrolling.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;

use crate::frame::FrameReason;
use crate::orchestration::scroll_containers;
use crate::runtime::{FoldInputs, ScrollTargets};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("scrolling.doc/scroll_into_view.md")]
    pub fn scroll_into_view(&mut self, node: NodeId) -> bool {
        let Some(target) = self.boxes.get(&node).copied() else {
            return false;
        };
        let Some(container) =
            ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
                .enclosing_scrollable(node)
        else {
            return false;
        };

        // Both boxes are in unscrolled layout space, so the node's offset within
        // the scrollable content is just the difference of their tops.
        let offset = target.origin().y - container.scrollport.origin.y;
        let height = target.size().height;
        let current = self.scroll.clamped(container);

        // The keyboard eats the bottom of the scrollport only insofar as the two
        // overlap: a container that ends above the keyboard loses nothing.
        let port_bottom = container.scrollport.origin.y + container.scrollport.size.height;
        let keyboard_top = self.surface.viewport().height - self.environment.keyboard.bottom;
        let occluded = (port_bottom - keyboard_top).clamp(0.0, container.scrollport.size.height);
        let visible = (container.scrollport.size.height - occluded).max(0.0);

        let desired = if offset < current {
            offset
        } else if offset + height > current + visible {
            offset + height - visible
        } else {
            return false;
        };
        let moved = self.scroll_by(container.node, desired - current);
        if (moved - current).abs() <= f32::EPSILON {
            return false;
        }
        self.request_frame(FrameReason::Input);
        true
    }

    #[doc = include_str!("scrolling.doc/scroll_top.md")]
    #[must_use]
    pub fn scroll_top(&self, node: NodeId) -> f32 {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        containers
            .iter()
            .find(|c| c.node == node)
            .map_or(0.0, |c| self.scroll.clamped(*c))
    }

    #[doc = include_str!("scrolling.doc/set_scroll_top.md")]
    pub fn set_scroll_top(&mut self, node: NodeId, px: f32) {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let Some(container) = containers.iter().find(|c| c.node == node).copied() else {
            return;
        };
        // `set` also refolds, which keeps the public viewport-relative box
        // projection in step with the new offset even before the caller's next
        // frame (spec §4.4).
        self.scroll.set(
            container,
            px,
            FoldInputs {
                tree: &self.tree,
                boxes: &self.boxes,
                containers: &containers,
            },
        );
    }

    #[doc = include_str!("scrolling.doc/scroll_by.md")]
    pub fn scroll_by(&mut self, node: NodeId, dy: f32) -> f32 {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let Some(container) = containers.iter().find(|c| c.node == node).copied() else {
            return 0.0;
        };
        // `shift` also refolds, which keeps the public viewport-relative box
        // projection in step with the new offset even before the caller's next
        // frame (spec §4.4).
        self.scroll.shift(
            container,
            dy,
            FoldInputs {
                tree: &self.tree,
                boxes: &self.boxes,
                containers: &containers,
            },
        )
    }

    #[doc = include_str!("scrolling.doc/handle_scroll.md")]
    pub fn handle_scroll(&mut self, delta_y: f32, position: Point) -> Option<NodeId> {
        // Only what the user may scroll. A `hidden` box is a scroll container in the
        // CSS sense — it clips and can be scrolled programmatically — but the wheel
        // must pass through it, or a row that clips a long name eats the scroll meant
        // for the list it sits in.
        let content = self.to_content_point(position);
        let target = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
            .scrollable_at(content)?;
        self.scroll_by(target, delta_y);
        self.request_frame(FrameReason::Input);
        Some(target)
    }

    #[doc = include_str!("scrolling.doc/to_content_point.md")]
    fn to_content_point(&self, position: Point) -> Point {
        self.surface.to_content_point(position)
    }
}
