#![doc = include_str!("../../../docs/api/runtime/ui_runtime/gestures.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_interaction::event::{EventOrigin, UiEvent};
use obscur_interaction::pointer::{PointerEvent, PointerEventKind};
use obscur_view::view::EVENT_LONG_PRESS;

use crate::frame::FrameReason;
use crate::runtime::{ActivePan, CommandDispatch, GestureArming};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/gestures.doc/arm_gestures.md")]
    pub(super) fn arm_gestures(
        &mut self,
        event: &PointerEvent,
        screen_point: Point,
        target: Option<NodeId>,
    ) {
        // The bundle borrows half the frame, so what it decided is taken out of it
        // before anything is armed: seating a hold asks for a frame, and that is a
        // `&mut self` the borrow would outlive.
        let (pan, hold) = {
            let arming = GestureArming::new(
                &self.tree,
                &self.computed,
                &self.boxes,
                &self.scroll,
                &self.virtual_lists,
            );
            (
                arming.pan_containers(event, screen_point),
                target.and_then(|node| arming.hold(event, screen_point, node)),
            )
        };
        if pan.iter().any(Option::is_some) {
            self.pointers
                .arm_pan(event.pointer(), screen_point, event.timestamp(), pan);
        }
        if let Some(hold) = hold {
            self.pointers.arm_hold(event.pointer(), hold);
            // Nothing else will schedule the frames that count the hold down.
            self.request_frame(FrameReason::Animation);
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/gestures.doc/update_pan.md")]
    pub(super) fn update_pan(
        &mut self,
        event: &PointerEvent,
        screen_point: Point,
        press_changed: &mut bool,
    ) -> bool {
        if event.kind() != PointerEventKind::Move {
            return false;
        }
        let Some(step) = self
            .pointers
            .update_pan(event.pointer(), screen_point, event.timestamp())
        else {
            return false;
        };
        if step.press_discarded {
            *press_changed = true;
        }
        self.default_won_touch_gesture(event.pointer());
        self.scroll_chain_from(step.container, step.scroll_delta);
        self.request_frame(FrameReason::Input);
        true
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/gestures.doc/advance_holds.md")]
    pub(super) fn advance_holds(&mut self, delta: f32) {
        for (pointer, hold) in self.pointers.advance_holds(delta) {
            self.default_won_touch_gesture(pointer);
            // §2.3 step 4: the default action runs only for a platform-vouched
            // gesture. An untrusted contact still had its hold counted and
            // observed; it just does not run the handler.
            if hold.trusted {
                match hold.row {
                    Some((list, index)) => {
                        self.virtual_lists.fire_row_hold(list, index);
                    }
                    None => {
                        self.fire_long_press(hold.target, hold.trusted);
                    }
                }
            }
            self.request_frame(FrameReason::Input);
        }
        if self.pointers.holds_pending() {
            self.request_frame(FrameReason::Animation);
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/gestures.doc/fire_long_press.md")]
    fn fire_long_press(&mut self, node: NodeId, trusted: bool) -> bool {
        let envelope = {
            let envelope = UiEvent::detached(EventOrigin::Gesture);
            if trusted {
                envelope.as_trusted()
            } else {
                envelope
            }
        };
        match CommandDispatch::binding_for(&self.tree, node, EVENT_LONG_PRESS) {
            Some((_bound, command)) => self.dispatch_command_event(Some(node), command, envelope),
            None => false,
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/gestures.doc/launch_fling.md")]
    pub(super) fn launch_fling(&mut self, active: &ActivePan, micros: u64) {
        if self.pointers.launch_fling(active, micros) {
            self.request_timeline_frame();
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/gestures.doc/advance_flings.md")]
    pub(super) fn advance_flings(&mut self, delta: f32) {
        let steps = self.pointers.advance_flings(delta);
        if steps.is_empty() {
            return;
        }
        let mut stalled = Vec::new();
        for ((node, axis), distance) in steps {
            let offset = if axis == 0 {
                Point::new(distance, 0.0)
            } else {
                Point::new(0.0, distance)
            };
            if distance != 0.0 && !self.scroll_chain_from(node, offset) {
                stalled.push((node, axis));
            }
        }
        self.pointers.settle_flings(&stalled);
        // Unconditionally, not just while a coast remains: the step that *ends* a
        // fling still moved the list, and skipping its frame would leave the last
        // few pixels of travel unpainted until something unrelated asked for a
        // frame.
        self.request_timeline_frame();
    }
}
