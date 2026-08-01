#![doc = include_str!("pointer.doc/_module.md")]

use obscur_data_transfer::drag::{DragSession, DragSource, DropTarget};
use obscur_identity::NodeId;
use obscur_interaction::pointer::PointerId;

use crate::frame::FrameReason;
use crate::orchestration::DragOutcome;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("pointer.doc/capture_pointer.md")]
    pub fn capture_pointer(&mut self, pointer: PointerId, node: NodeId) -> bool {
        if self.tree.get(node).is_none() {
            return false;
        }
        self.pointers.capture(pointer, node);
        true
    }

    #[doc = include_str!("pointer.doc/pointer_capture.md")]
    #[must_use]
    pub fn pointer_capture(&self, pointer: PointerId) -> Option<NodeId> {
        self.pointers
            .capture_target(pointer)
            .filter(|node| self.tree.get(*node).is_some())
    }

    #[doc = include_str!("pointer.doc/release_pointer_capture.md")]
    pub fn release_pointer_capture(&mut self, pointer: PointerId) -> bool {
        self.pointers.release_capture(pointer)
    }

    #[doc = include_str!("pointer.doc/begin_drag.md")]
    pub fn begin_drag(&mut self, source: DragSource) -> bool {
        let started = self.drag.begin(source);
        if started {
            self.request_frame(FrameReason::Input);
        }
        started
    }

    #[doc = include_str!("pointer.doc/finish_drag.md")]
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

    #[doc = include_str!("pointer.doc/register_drop_target.md")]
    pub fn register_drop_target(&mut self, target: DropTarget) {
        self.drag.register_target(target);
    }

    #[doc = include_str!("pointer.doc/unregister_drop_target.md")]
    pub fn unregister_drop_target(&mut self, node: NodeId) {
        self.drag.unregister_target(node);
    }

    #[doc = include_str!("pointer.doc/drag_session.md")]
    #[must_use]
    pub fn drag_session(&self) -> Option<&DragSession> {
        self.drag.session()
    }

    #[doc = include_str!("pointer.doc/take_drag_outcome.md")]
    #[must_use]
    pub fn take_drag_outcome(&mut self) -> Option<DragOutcome> {
        self.drag_outcome.take()
    }
}
