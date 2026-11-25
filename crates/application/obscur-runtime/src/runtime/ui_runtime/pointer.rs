#![doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer.doc/_module.md")]

use obscur_data_transfer::drag::{DragSession, DragSource, DropTarget};
use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_interaction::pointer::{PointerButtons, PointerId};

use crate::frame::FrameReason;
use crate::orchestration::{DragOutcome, scroll_containers};
use crate::runtime::{HitTesting, ScrollTargets};

use super::UiRuntime;

impl UiRuntime {
    /// Returns the held buttons from the latest mouse event.
    #[must_use]
    pub const fn mouse_buttons(&self) -> PointerButtons {
        self.pointers.mouse_buttons
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer.doc/capture_pointer.md")]
    pub fn capture_pointer(&mut self, pointer: PointerId, node: NodeId) -> bool {
        if self.tree.get(node).is_none() {
            return false;
        }
        self.pointers.capture(pointer, node);
        true
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer.doc/pointer_capture.md")]
    #[must_use]
    pub fn pointer_capture(&self, pointer: PointerId) -> Option<NodeId> {
        self.pointers
            .capture_target(pointer)
            .filter(|node| self.tree.get(*node).is_some())
    }

    /// Releases any capture held on `pointer`, returning whether one was held
    /// (spec §2.4).
    pub fn release_pointer_capture(&mut self, pointer: PointerId) -> bool {
        let released = self.pointers.release_capture(pointer);
        self.finish_scrollbar_drag(pointer);
        released
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer.doc/begin_drag.md")]
    pub fn begin_drag(&mut self, source: DragSource) -> bool {
        let started = self.drag.begin(source);
        if started {
            self.drag_scroll_point = None;
            self.selection_scroll_point = None;
            self.request_frame(FrameReason::Input);
        }
        started
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer.doc/finish_drag.md")]
    pub(super) fn finish_drag(&mut self) {
        let version = self.enactor.version();
        // The enactor's own validator: a drop is checked against the same rules a
        // reconcile is, not a second set this path invented.
        let Some(outcome) = self.drag.release(self.enactor.validator(), version) else {
            return;
        };
        match &outcome {
            DragOutcome::Dropped { commit, .. } => {
                self.apply_transaction(commit);
            }
            DragOutcome::Cancelled => {
                // Nothing was applied — but the drag's visuals still have to go.
                self.request_frame(FrameReason::Input);
            }
        }
        self.drag_outcome = Some(outcome);
    }

    /// Arms ticking only while a drag pointer is near an eligible scrollport edge.
    pub(super) fn arm_drag_autoscroll(&mut self, point: Point) {
        let target = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
            .scrollable_at(point, Point::new(1.0, 1.0));
        self.drag_scroll_point = target
            .filter(|&node| self.edge_scroll_velocity(node, point) != Point::ORIGIN)
            .map(|_| point);
        if self.drag_scroll_point.is_some() {
            self.request_timeline_frame();
        }
    }

    /// Scrolls under a stationary drag pointer at the edge and refreshes the drop target.
    pub(super) fn advance_drag_autoscroll(&mut self, delta: f32) {
        if !delta.is_finite() || delta <= 0.0 {
            return;
        }
        let Some(point) = self
            .drag_scroll_point
            .take()
            .filter(|_| self.drag.is_active())
        else {
            return;
        };
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let targets = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll);
        let Some(scroller) = targets.scrollable_in(point, Point::new(1.0, 1.0), &containers) else {
            return;
        };
        if !self.scroll_near_edge(scroller, point, delta, &containers) {
            return;
        }

        let scrolls = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
            .paint(&self.computed)
            .into_iter()
            .map(|(node, paint)| (node, paint.offset()))
            .collect();
        let target = self
            .hits
            .resolve(&self.tree, &self.computed, &self.boxes, point, &scrolls);
        let position = target.map_or(point, |node| {
            HitTesting::content_point(&self.tree, &scrolls, node, point)
        });
        self.drag.pointer_moved(position, target);
        self.drag_scroll_point = Some(point);
        self.request_timeline_frame();
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer.doc/register_drop_target.md")]
    pub fn register_drop_target(&mut self, target: DropTarget) {
        self.drag.register_target(target);
    }

    /// Removes `node`'s drop-target registration, if any.
    pub fn unregister_drop_target(&mut self, node: NodeId) {
        self.drag.unregister_target(node);
    }

    /// The live drag session, for a host drawing the drag; [`None`] when none is
    /// in flight.
    #[must_use]
    pub fn drag_session(&self) -> Option<&DragSession> {
        self.drag.session()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer.doc/take_drag_outcome.md")]
    #[must_use]
    pub fn take_drag_outcome(&mut self) -> Option<DragOutcome> {
        self.drag_outcome.take()
    }
}
