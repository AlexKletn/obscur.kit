#![doc = include_str!("../../../docs/api/runtime/ui_runtime/scrolling.doc/_module.md")]

use obscur_data_transfer::drag::DragAutoscroll;
use obscur_geometry::{Point, Size};
use obscur_identity::{FastMap, InternedSymbol, NodeId};
use obscur_interaction::event::UiEvent;
use obscur_interaction::pointer::WheelEvent;
use obscur_style::property::{PropertyId, PropertyValue};
use obscur_view::view::EVENT_WHEEL;

use crate::frame::FrameReason;
use crate::orchestration::{
    ComputedStyles, ScrollContainer, establishes_scroll_container, overflow_padding_box,
    overflow_scrolls, overflow_scrolls_x, scroll_containers,
};
use crate::runtime::{
    CommandDispatch, FoldInputs, HitTesting, ScrollAnchor, ScrollMetrics, ScrollState,
    ScrollTargets,
};

use crate::runtime::scroll_snap::{SNAP_DURATION, ScrollSnap};

use super::UiRuntime;

impl UiRuntime {
    /// Reports the current offset, viewport size, and content extent of an arranged container with a nonzero scroll range.
    #[must_use]
    pub fn scroll_metrics(&self, node: NodeId) -> Option<ScrollMetrics> {
        let container = scroll_containers(&self.tree, &self.computed, &self.boxes)
            .into_iter()
            .find(|container| container.node == node)?;
        Some(ScrollMetrics {
            offset: self.scroll.offset(container),
            viewport: container.scrollport.size,
            content_extent: Size::new(container.content_width, container.content_height),
        })
    }

    /// Reports displacement per second sampled on the last positive runtime motion tick. Wheel, touch, and timed scrolling contribute to the sample. An idle tick or an immediate offset jump clears it.
    #[must_use]
    pub fn scroll_velocity(&self, node: NodeId) -> Option<Point> {
        self.scroll_metrics(node)
            .map(|_| self.scroll.velocity(node))
    }

    /// Captures both axes of a laid-out scroll container for application-owned storage.
    #[must_use]
    pub fn capture_scroll_state(&self, node: NodeId) -> Option<ScrollState> {
        self.scroll_metrics(node)
            .map(|metrics| ScrollState::capture(metrics.offset))
    }

    /// Restores a persisted position to a laid-out container and requests a frame.
    /// Returns false for an unknown format version, non-finite coordinates or a
    /// missing scroll container.
    pub fn restore_scroll_state(&mut self, node: NodeId, state: ScrollState) -> bool {
        let Some(offset) = state.offset() else {
            return false;
        };
        self.scroll_to(node, offset)
    }

    /// Captures a descendant’s viewport-relative layout position before a reflow. The descendant must belong to the container’s visual scroll content.
    #[must_use]
    pub fn capture_scroll_anchor(&self, container: NodeId, node: NodeId) -> Option<ScrollAnchor> {
        let position = self.scroll_anchor_position(container, node, &self.computed)?;
        let offset = self.scroll_metrics(container)?.offset;
        Some(ScrollAnchor {
            container,
            node,
            viewport_position: Point::new(position.x - offset.x, position.y - offset.y),
        })
    }

    /// Restores a captured anchor after layout. The same container and descendant must still be present. The resulting offset is clamped to the current range.
    pub fn restore_scroll_anchor(&mut self, anchor: ScrollAnchor) -> bool {
        let Some(position) =
            self.scroll_anchor_position(anchor.container, anchor.node, &self.computed)
        else {
            return false;
        };
        self.scroll_to(
            anchor.container,
            Point::new(
                position.x - anchor.viewport_position.x,
                position.y - anchor.viewport_position.y,
            ),
        )
    }

    /// Tracks an arranged visual descendant across subsequent reflows. Pass None to clear the anchor. The runtime compensates layout movement within the available scroll range.
    pub fn set_scroll_anchor(&mut self, container: NodeId, node: Option<NodeId>) -> bool {
        let Some(node) = node else {
            return self.tracked_scroll_anchors.remove(&container).is_some();
        };
        if self.capture_scroll_anchor(container, node).is_none() {
            return false;
        }
        self.tracked_scroll_anchors.insert(container, node);
        true
    }

    /// Captures tracked anchor positions after input and before frame layout.
    pub(super) fn capture_tracked_scroll_anchors(&self) -> Vec<(ScrollAnchor, Point)> {
        if self.tracked_scroll_anchors.is_empty() {
            return Vec::new();
        }
        scroll_containers(&self.tree, &self.computed, &self.boxes)
            .into_iter()
            .filter_map(|container| {
                let &node = self.tracked_scroll_anchors.get(&container.node)?;
                let position = self.scroll_anchor_position(container.node, node, &self.computed)?;
                let offset = self.scroll.offset(container);
                Some((
                    ScrollAnchor {
                        container: container.node,
                        node,
                        viewport_position: Point::new(position.x - offset.x, position.y - offset.y),
                    },
                    position,
                ))
            })
            .collect()
    }

    /// Compensates layout movement before compose and removes anchors whose nodes exited.
    pub(super) fn restore_tracked_scroll_anchors(
        &mut self,
        anchors: Vec<(ScrollAnchor, Point)>,
        computed: &ComputedStyles,
    ) {
        self.tracked_scroll_anchors
            .retain(|container, node| self.tree.contains(*container) && self.tree.contains(*node));
        if anchors.is_empty() {
            return;
        }
        let containers = scroll_containers(&self.tree, computed, &self.boxes);
        for (anchor, previous) in anchors {
            let Some(position) =
                self.scroll_anchor_position(anchor.container, anchor.node, computed)
            else {
                continue;
            };
            if position == previous {
                continue;
            }
            let Some(container) = containers
                .iter()
                .find(|container| container.node == anchor.container)
            else {
                continue;
            };
            self.scroll.restore_unfolded(
                *container,
                Point::new(
                    position.x - anchor.viewport_position.x,
                    position.y - anchor.viewport_position.y,
                ),
            );
        }
    }

    /// Finds a node’s unscrolled layout position relative to its containing scrollport.
    fn scroll_anchor_position(
        &self,
        container: NodeId,
        node: NodeId,
        computed: &ComputedStyles,
    ) -> Option<Point> {
        let origin = self.boxes.get(&node)?.origin();
        let viewport =
            overflow_padding_box(computed.get(&container), self.boxes.get(&container)?.bounds)
                .origin;
        let mut current = node;
        for _ in 0..self.tree.len() {
            let parent = self.tree.visual_parent_of(current)?;
            if parent == container {
                return Some(Point::new(origin.x - viewport.x, origin.y - viewport.y));
            }
            if establishes_scroll_container(computed, parent) {
                break;
            }
            current = parent;
        }
        None
    }

    /// Sets both scroll axes in one update and clamps the offset to the current range.
    /// Returns whether the request was accepted. The container must have a nonzero
    /// scroll range. Coordinates are logical pixels.
    /// A right-to-left container uses nonpositive horizontal offsets.
    pub fn scroll_to(&mut self, node: NodeId, offset: Point) -> bool {
        if !offset.x.is_finite() || !offset.y.is_finite() {
            return false;
        }
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let Some(container) = containers.iter().find(|container| container.node == node) else {
            return false;
        };
        self.cancel_scroll_snap(node);
        self.pointers.stop_flings(node);
        self.scroll.restore(
            *container,
            offset,
            FoldInputs {
                tree: &self.tree,
                boxes: &self.boxes,
                containers: &containers,
            },
        );
        self.request_frame(FrameReason::UiCommit);
        true
    }

    /// Configures positions to snap to after user scrolling settles. An empty slice clears the policy.
    pub fn set_scroll_snap_points(&mut self, node: NodeId, candidates: &[Point]) -> bool {
        if candidates.is_empty() {
            return self.scroll_snaps.remove(&node).is_some();
        }
        if !self.tree.contains(node)
            || candidates
                .iter()
                .any(|point| !point.x.is_finite() || !point.y.is_finite())
        {
            return false;
        }
        self.scroll_snaps
            .insert(node, ScrollSnap::new(candidates.to_vec()));
        true
    }

    /// Starts snapping after the quiet period and after touch inertia ends.
    pub(super) fn advance_scroll_snaps(&mut self, delta: f32) {
        if !delta.is_finite()
            || delta <= 0.0
            || self.pointers.has_active_pan()
            || self.pointers.has_flings()
        {
            return;
        }
        let due: Vec<_> = self
            .scroll_snaps
            .iter_mut()
            .filter_map(|(&node, snap)| {
                snap.advance(delta).then(|| (node, snap.candidates.clone()))
            })
            .collect();
        for (node, candidates) in due {
            self.snap_scroll_to_nearest(node, &candidates, SNAP_DURATION);
        }
    }

    /// Cancels any pending snap for the container before a programmatic scroll.
    fn cancel_scroll_snap(&mut self, node: NodeId) {
        if let Some(snap) = self.scroll_snaps.get_mut(&node) {
            snap.cancel();
        }
    }

    /// Scrolls to the nearest application-supplied candidate in logical pixels. Candidates are clamped to the current scroll range before comparison. Nonfinite candidates are skipped, and ties favor the first candidate. Pass zero seconds for immediate snapping.
    pub fn snap_scroll_to_nearest(
        &mut self,
        node: NodeId,
        candidates: &[Point],
        duration: f32,
    ) -> bool {
        let container = scroll_containers(&self.tree, &self.computed, &self.boxes)
            .into_iter()
            .find(|container| container.node == node);
        let Some(container) = container else {
            return false;
        };
        let current = self.scroll.offset(container);
        let distance_squared = |point: Point| {
            let dx = f64::from(point.x) - f64::from(current.x);
            let dy = f64::from(point.y) - f64::from(current.y);
            dx * dx + dy * dy
        };
        let target = candidates
            .iter()
            .filter(|point| point.x.is_finite() && point.y.is_finite())
            .map(|point| {
                Point::new(
                    container.clamp_left(point.x),
                    point.y.clamp(0.0, container.max_scroll_top()),
                )
            })
            .min_by(|a, b| distance_squared(*a).total_cmp(&distance_squared(*b)));
        target.is_some_and(|offset| self.scroll_to_smooth(node, offset, duration))
    }

    /// Animates both axes to a clamped offset over the duration in seconds.
    /// A zero duration scrolls immediately. A new request replaces the motion for
    /// that container, and a direct scroll update interrupts it.
    pub fn scroll_to_smooth(&mut self, node: NodeId, offset: Point, duration: f32) -> bool {
        if !offset.x.is_finite() || !offset.y.is_finite() || !duration.is_finite() || duration < 0.0
        {
            return false;
        }
        if duration == 0.0 {
            return self.scroll_to(node, offset);
        }
        let Some(container) = scroll_containers(&self.tree, &self.computed, &self.boxes)
            .into_iter()
            .find(|container| container.node == node)
        else {
            return false;
        };
        self.cancel_scroll_snap(node);
        self.pointers.stop_flings(node);
        self.scroll.animate(container, offset, duration);
        if self.scroll.has_motion() {
            self.request_timeline_frame();
        }
        true
    }

    /// Advances every active programmatic scroll on the runtime motion clock.
    pub(super) fn advance_smooth_scrolls(&mut self, delta: f32) {
        if !self.scroll.has_motion() || !delta.is_finite() || delta <= 0.0 {
            return;
        }
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        if self.scroll.advance_motion(
            delta,
            FoldInputs {
                tree: &self.tree,
                boxes: &self.boxes,
                containers: &containers,
            },
        ) {
            self.request_timeline_frame();
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/scrolling.doc/WHEEL_LINE_STEP.md")]
    pub const WHEEL_LINE_STEP: f32 = 48.0;

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/scrolling.doc/scroll_into_view.md")]
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
        let current = self.scroll.offset(container);

        // The keyboard eats the bottom of the scrollport only insofar as the two
        // overlap: a container that ends above the keyboard loses nothing.
        let port_bottom = container.scrollport.origin.y + container.scrollport.size.height;
        let keyboard_top = self.surface.viewport().height - self.environment.keyboard.bottom;
        let occluded = (port_bottom - keyboard_top).clamp(0.0, container.scrollport.size.height);
        let visible = (container.scrollport.size.height - occluded).max(0.0);

        let reveal = |start: f32, extent: f32, current: f32, visible: f32| {
            if start < current {
                start - current
            } else if start + extent > current + visible {
                start + extent - current - visible
            } else {
                0.0
            }
        };
        let delta = Point::new(
            reveal(
                target.origin().x - container.scrollport.origin.x,
                target.size().width,
                current.x,
                container.scrollport.size.width,
            ),
            reveal(offset, height, current.y, visible),
        );
        let moved = self.scroll_user_by(container.node, delta);
        if moved {
            self.request_frame(FrameReason::Input);
        }
        moved
    }

    /// Return the clamped horizontal scroll offset in logical pixels. In horizontal RTL, zero is at the right edge and scrolling left yields negative values.
    #[must_use]
    pub fn scroll_left(&self, node: NodeId) -> f32 {
        scroll_containers(&self.tree, &self.computed, &self.boxes)
            .into_iter()
            .find(|c| c.node == node)
            .map_or(0.0, |c| self.scroll.offset(c).x)
    }

    /// Set the signed horizontal scroll offset in logical pixels and schedule a frame. Horizontal RTL accepts offsets from the negative extent to zero.
    pub fn set_scroll_left(&mut self, node: NodeId, px: f32) {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        if let Some(container) = containers.iter().find(|c| c.node == node) {
            if px.is_finite() {
                self.cancel_scroll_snap(node);
            }
            self.scroll.set_left(
                *container,
                px,
                FoldInputs {
                    tree: &self.tree,
                    boxes: &self.boxes,
                    containers: &containers,
                },
            );
            self.request_frame(FrameReason::Input);
        }
    }

    pub(super) fn scroll_user_by(&mut self, node: NodeId, mut delta: Point) -> bool {
        if !overflow_scrolls_x(&self.computed, node) {
            delta.x = 0.0;
        }
        if !overflow_scrolls(&self.computed, node) {
            delta.y = 0.0;
        }
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let Some(container) = containers.iter().find(|c| c.node == node) else {
            return false;
        };
        let before = self.scroll.offset(*container);
        let after = self.scroll.shift_offset(
            *container,
            delta,
            FoldInputs {
                tree: &self.tree,
                boxes: &self.boxes,
                containers: &containers,
            },
        );
        before != after
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/scrolling.doc/scroll_top.md")]
    #[must_use]
    pub fn scroll_top(&self, node: NodeId) -> f32 {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        containers
            .iter()
            .find(|c| c.node == node)
            .map_or(0.0, |c| self.scroll.clamped(*c))
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/scrolling.doc/set_scroll_top.md")]
    pub fn set_scroll_top(&mut self, node: NodeId, px: f32) {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let Some(container) = containers.iter().find(|c| c.node == node).copied() else {
            return;
        };
        // `set` also refolds, which keeps the public viewport-relative box
        // projection in step with the new offset even before the caller's next
        // frame (spec §4.4).
        if px.is_finite() {
            self.cancel_scroll_snap(node);
        }
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

    /// Scrolls the scroll container `node` by `dy` logical px (positive = toward the
    /// content's end), clamped to the scrollable range, and returns the resulting
    /// `scroll_top`. A `node` that is not an overflowing scroll container is
    /// ignored and `0` is returned (CSS Overflow 3 §scrolling).
    pub fn scroll_by(&mut self, node: NodeId, dy: f32) -> f32 {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let Some(container) = containers.iter().find(|c| c.node == node).copied() else {
            return 0.0;
        };
        // `shift` also refolds, which keeps the public viewport-relative box
        // projection in step with the new offset even before the caller's next
        // frame (spec §4.4).
        if dy.is_finite() {
            self.cancel_scroll_snap(node);
        }
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

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/scrolling.doc/handle_scroll.md")]
    pub fn handle_scroll(&mut self, delta_y: f32, position: Point) -> Option<NodeId> {
        self.handle_wheel(WheelEvent::new(position, delta_y))
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/scrolling.doc/handle_wheel.md")]
    pub fn handle_wheel(&mut self, wheel: WheelEvent) -> Option<NodeId> {
        let content = self.to_content_point(wheel.position());
        // Lines become pixels HERE and not in the platform adapter. How far a
        // notch scrolls is this toolkit's policy, not winit's fact, and the
        // adapter that used to multiply by 48 before the runtime saw anything
        // both hid the policy where nobody would look for it and destroyed the
        // one thing a handler might want to treat differently.
        let scale = self.surface.camera_scale();
        let delta = Point::new(
            wheel.pixels_x(Self::WHEEL_LINE_STEP) / scale,
            wheel.pixels_y(Self::WHEEL_LINE_STEP) / scale,
        );

        // 1. The raw `wheel` event (§2.3), dispatched to the node under the point
        //    BEFORE the scroll it can refuse. Same ordering as `pointerdown` and
        //    `keydown`, and for the same reason: a wheel delivered after the
        //    container had already moved could report the notch but never
        //    decline it. A refusal is a claim — the node that refused is returned
        //    as the node that took the event, so the host's own `on_scroll`
        //    fallback does not then run as if nothing had wanted it.
        if let Some(claimed) = self.dispatch_wheel(wheel, content) {
            self.refuse_scroll_candidate(wheel);
            return Some(claimed);
        }
        if let Some(claimed) = self.arbitrate_scroll_candidate(wheel, content) {
            return Some(claimed);
        }

        // 2. Only what the user may scroll. A `hidden` box is a scroll container in
        //    the CSS sense — it clips and can be scrolled programmatically — but the
        //    wheel must pass through it, or a row that clips a long name eats the
        //    scroll meant for the list it sits in.

        let target = self.scroll_wheel_chain(content, delta);
        if target.is_some()
            && delta.x.is_finite()
            && delta.y.is_finite()
            && self.refresh_hover_at_cursor()
        {
            self.request_frame(FrameReason::Input);
        }
        target
    }

    /// Passes unconsumed wheel movement up the visual scroll ancestry and refolds once.
    fn scroll_wheel_chain(&mut self, content: Point, delta: Point) -> Option<NodeId> {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let target = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
            .scrollable_in(content, delta, &containers)?;
        let moved = self.apply_scroll_chain(target, delta, &containers);
        if moved.is_some() {
            self.request_frame(FrameReason::Input);
        }
        moved.or(Some(target))
    }

    /// Applies a delta through the visual scroll chain and reports whether any container moved.
    pub(super) fn scroll_chain_from(&mut self, target: NodeId, delta: Point) -> bool {
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        self.apply_scroll_chain(target, delta, &containers)
            .is_some()
    }

    /// Applies time-based scrolling for a pointer near the viewport boundary.
    pub(super) fn scroll_near_edge(
        &mut self,
        scroller: NodeId,
        point: Point,
        delta: f32,
        containers: &[ScrollContainer],
    ) -> bool {
        let velocity = self.edge_scroll_velocity(scroller, point);
        self.apply_scroll_chain(
            scroller,
            Point::new(velocity.x * delta, velocity.y * delta),
            containers,
        )
        .is_some()
    }

    /// Returns the scrolling velocity in logical pixels per second for a viewport point.
    pub(super) fn edge_scroll_velocity(&self, scroller: NodeId, point: Point) -> Point {
        let Some(port) = self
            .scroll
            .projection(&self.boxes)
            .get(&scroller)
            .map(|layout| overflow_padding_box(self.computed.get(&scroller), layout.bounds))
        else {
            return Point::ORIGIN;
        };
        let margin = (port.size.width.min(port.size.height) * 0.5).min(32.0);
        if margin <= 0.0 {
            return Point::ORIGIN;
        }
        let local = Point::new(point.x - port.origin.x, point.y - port.origin.y);
        let (x, y) = DragAutoscroll::new(margin, 1200.0).velocity(local, port.size);
        Point::new(x, y)
    }

    /// Applies scroll movement from a fixed target up its visual ancestry.
    fn apply_scroll_chain(
        &mut self,
        target: NodeId,
        delta: Point,
        containers: &[ScrollContainer],
    ) -> Option<NodeId> {
        let finite = |delta: f32| if delta.is_finite() { delta } else { 0.0 };
        let mut remaining = Point::new(finite(delta.x), finite(delta.y));
        let mut first_moved = None;
        let mut virtual_window_changed = false;
        let mut current = Some(target);
        for _ in 0..self.tree.len() {
            let Some(node) = current else { break };
            current = self.tree.visual_parent_of(node);
            let Some(container) = containers.iter().find(|container| container.node == node) else {
                continue;
            };
            let offered = Point::new(
                if overflow_scrolls_x(&self.computed, node) {
                    remaining.x
                } else {
                    0.0
                },
                if overflow_scrolls(&self.computed, node) {
                    remaining.y
                } else {
                    0.0
                },
            );
            if offered == Point::ORIGIN {
                continue;
            }
            let before = self.scroll.offset(*container);
            let after = self.scroll.shift_unfolded(*container, offered);
            if before != after {
                first_moved.get_or_insert(node);
                virtual_window_changed |= before.y != after.y
                    && self.virtual_lists.window_changes_at(
                        node,
                        after.y,
                        self.surface.viewport().height,
                    );
                if !self.drag.is_active()
                    && self.pointers.text_drag().is_none()
                    && let Some(snap) = self.scroll_snaps.get_mut(&node)
                {
                    snap.arm();
                }
            }
            let style = self.computed.get(&node);
            let unconsumed = |requested: f32, consumed: f32, property| {
                if matches!(
                    style.and_then(|style| style.get(property)).map(|value| value.value()),
                    Some(PropertyValue::Keyword(keyword))
                        if *keyword == InternedSymbol::named("contain")
                            || *keyword == InternedSymbol::named("none")
                ) {
                    return 0.0;
                }
                (requested - consumed).clamp(requested.min(0.0), requested.max(0.0))
            };
            remaining = Point::new(
                unconsumed(
                    remaining.x,
                    after.x - before.x,
                    PropertyId::OverscrollBehaviorX,
                ),
                unconsumed(
                    remaining.y,
                    after.y - before.y,
                    PropertyId::OverscrollBehaviorY,
                ),
            );
            if remaining == Point::ORIGIN {
                break;
            }
        }
        if first_moved.is_some() {
            self.scroll.refold(FoldInputs {
                tree: &self.tree,
                boxes: &self.boxes,
                containers,
            });
        }
        if virtual_window_changed {
            self.request_frame(FrameReason::UiCommit);
        }
        first_moved
    }

    pub(super) fn point_event_binding(
        &mut self,
        content: Point,
        event_name: InternedSymbol,
    ) -> Option<(NodeId, InternedSymbol, Point)> {
        let scrolls: FastMap<NodeId, Point> =
            ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
                .paint(&self.computed)
                .into_iter()
                .map(|(node, paint)| (node, paint.offset()))
                .collect();
        let target =
            self.hits
                .resolve(&self.tree, &self.computed, &self.boxes, content, &scrolls)?;
        let folded = HitTesting::content_point(&self.tree, &scrolls, target, content);
        let (node, command) = CommandDispatch::binding_for(&self.tree, target, event_name)?;
        Some((node, command, folded))
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/scrolling.doc/dispatch_wheel.md")]
    fn dispatch_wheel(&mut self, wheel: WheelEvent, content: Point) -> Option<NodeId> {
        // Cost, and the same gate the raw pointer names use. A trackpad's
        // two-finger scroll arrives as pixel deltas at nearly the rate its moves
        // do, and everything below — folding the point, hit-testing it, walking
        // the target's ancestors — is skipped rather than merely fruitless for a
        // tree that binds nothing.
        if !self.pointer_bindings.binds(EVENT_WHEEL) {
            return None;
        }
        // The node under the point, resolved exactly as a click's would be: the
        // content point plus every scroll container's offset, which the hit test
        // folds back per candidate (a fixed box rides none of them).
        // Deliberately NOT the scroll container that would take the notch — that
        // container is normally an ancestor of the hit node, and the ancestor
        // walk below reaches it anyway, so targeting it directly would only rob a
        // row of the chance to hear its own wheel.
        let (node, command, folded) = self.point_event_binding(content, EVENT_WHEEL)?;
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

    /// Maps a host `position` (physical space) to content space via the inverse of
    /// the root render transform `scale(s) · translate(−ox, −oy)` — the same
    /// mapping [`handle_pointer`](Self::handle_pointer) applies before hit-testing.
    fn to_content_point(&self, position: Point) -> Point {
        self.surface.to_content_point(position)
    }
}
