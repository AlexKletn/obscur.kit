#![doc = include_str!("scrolling.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_interaction::event::UiEvent;
use obscur_interaction::pointer::WheelEvent;
use obscur_view::view::EVENT_WHEEL;

use crate::frame::FrameReason;
use crate::orchestration::scroll_containers;
use crate::runtime::{CommandDispatch, FoldInputs, ScrollTargets};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("scrolling.doc/WHEEL_LINE_STEP.md")]
    pub const WHEEL_LINE_STEP: f32 = 48.0;

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
        self.handle_wheel(WheelEvent::new(position, delta_y))
    }

    #[doc = include_str!("scrolling.doc/handle_wheel.md")]
    pub fn handle_wheel(&mut self, wheel: WheelEvent) -> Option<NodeId> {
        let content = self.to_content_point(wheel.position());
        // Lines become pixels HERE and not in the platform adapter. How far a
        // notch scrolls is this toolkit's policy, not winit's fact, and the
        // adapter that used to multiply by 48 before the runtime saw anything
        // both hid the policy where nobody would look for it and destroyed the
        // one thing a handler might want to treat differently.
        let delta_y = wheel.pixels_y(Self::WHEEL_LINE_STEP);

        // 1. The raw `wheel` event (§2.3), dispatched to the node under the point
        //    BEFORE the scroll it can refuse. Same ordering as `pointerdown` and
        //    `keydown`, and for the same reason: a wheel delivered after the
        //    container had already moved could report the notch but never
        //    decline it. A refusal is a claim — the node that refused is returned
        //    as the node that took the event, so the host's own `on_scroll`
        //    fallback does not then run as if nothing had wanted it.
        if let Some(claimed) = self.dispatch_wheel(wheel, content) {
            return Some(claimed);
        }

        // 2. Only what the user may scroll. A `hidden` box is a scroll container in
        //    the CSS sense — it clips and can be scrolled programmatically — but the
        //    wheel must pass through it, or a row that clips a long name eats the
        //    scroll meant for the list it sits in.
        //
        //    Vertical only: `delta_x` reaches a handler and nothing else, because
        //    `ScrollOffsets` holds one axis. That is a limit of the scroll model,
        //    not of the event.
        let target = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
            .scrollable_at(content)?;
        self.scroll_by(target, delta_y);
        self.request_frame(FrameReason::Input);
        Some(target)
    }

    #[doc = include_str!("scrolling.doc/dispatch_wheel.md")]
    fn dispatch_wheel(&mut self, wheel: WheelEvent, content: Point) -> Option<NodeId> {
        // Cost, and the same gate the raw pointer names use. A trackpad's
        // two-finger scroll arrives as pixel deltas at nearly the rate its moves
        // do, and everything below — folding the point, hit-testing it, walking
        // the target's ancestors — is skipped rather than merely fruitless for a
        // tree that binds nothing.
        if !self.pointer_bindings.binds(EVENT_WHEEL) {
            return None;
        }
        // The node under the point, resolved exactly as a click's would be: fold
        // the content point through the scroll containers it lies within, then
        // hit-test in both spaces (a clip is tested on screen, a box in
        // unscrolled layout space). Deliberately NOT the scroll container that
        // would take the notch — that container is normally an ancestor of the
        // hit node, and the ancestor walk below reaches it anyway, so targeting
        // it directly would only rob a row of the chance to hear its own wheel.
        let folded = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
            .fold_point(content);
        let target = self
            .hits
            .resolve(&self.tree, &self.computed, &self.boxes, folded, content)?;
        let (node, command) = CommandDispatch::binding_for(&self.tree, target, EVENT_WHEEL)?;
        // Aimed at the FOLDED point so a listener and a hit test agree about
        // where the notch landed; everything else — both deltas, the unit, the
        // clock, the trust — is the platform's own report, untouched.
        let prevented = self
            .dispatch_command_outcome(Some(node), command, UiEvent::from_wheel(wheel.at(folded)))
            .default_prevented;
        // Only a refusal claims the notch. A listener that merely watched it gets
        // the scroll it would have got anyway.
        prevented.then_some(node)
    }

    #[doc = include_str!("scrolling.doc/to_content_point.md")]
    fn to_content_point(&self, position: Point) -> Point {
        self.surface.to_content_point(position)
    }
}
