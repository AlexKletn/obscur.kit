use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::UiEvent;
use obscur_interaction::gesture::{GesturePhase, TouchpadGestureDelta, TouchpadGestureEvent};
use obscur_view::view::EVENT_TOUCHPAD_GESTURE;

use super::UiRuntime;

#[derive(Debug, Clone, Copy)]
pub(super) struct NativeTouchpadGesture {
    node: NodeId,
    command: InternedSymbol,
    last: TouchpadGestureEvent,
}

impl UiRuntime {
    /// Delivers native touchpad input from the window host. The initial physical window anchor resolves a binding and logical position that remain fixed through completion or cancellation.
    pub fn handle_touchpad_gesture(&mut self, mut event: TouchpadGestureEvent) -> bool {
        if !event.trust.is_trusted() {
            return false;
        }
        if event.phase == GesturePhase::Started {
            self.cancel_native_touchpad_gesture();
            if !self.pointer_bindings.binds(EVENT_TOUCHPAD_GESTURE) {
                return false;
            }
            let Some((node, command, position)) = self.point_event_binding(
                self.surface.to_content_point(event.position),
                EVENT_TOUCHPAD_GESTURE,
            ) else {
                return false;
            };
            if matches!(event.delta, TouchpadGestureDelta::Swipe(_)) {
                self.suspend_scroll_candidate();
            }
            event.position = position;
            self.native_touchpad_gesture = Some(NativeTouchpadGesture {
                node,
                command,
                last: event,
            });
        }
        let Some(mut session) = self.native_touchpad_gesture.take() else {
            return false;
        };
        if !self.tree.contains(session.node) {
            return false;
        }
        if std::mem::discriminant(&event.delta) != std::mem::discriminant(&session.last.delta) {
            self.native_touchpad_gesture = Some(session);
            return false;
        }
        event.position = session.last.position;
        event.finger_count = session.last.finger_count;
        session.last = event;
        if matches!(event.phase, GesturePhase::Started | GesturePhase::Updated) {
            self.native_touchpad_gesture = Some(session);
        }
        self.dispatch_command_event(
            Some(session.node),
            session.command,
            UiEvent::from_touchpad_gesture(event),
        )
    }

    /// Cancels active native touchpad and phased-scroll candidates. Hosts call this when the window loses focus or its input device is removed.
    pub fn cancel_indirect_gestures(&mut self) {
        self.suspend_scroll_candidate();
        self.cancel_native_touchpad_gesture();
    }

    pub(super) fn retain_live_indirect_gestures(&mut self) {
        if self
            .native_touchpad_gesture
            .is_some_and(|session| !self.tree.contains(session.node))
        {
            self.native_touchpad_gesture = None;
        }
        self.retain_live_scroll_candidate();
    }

    fn cancel_native_touchpad_gesture(&mut self) {
        if let Some(session) = self.native_touchpad_gesture {
            self.handle_touchpad_gesture(TouchpadGestureEvent {
                phase: GesturePhase::Cancelled,
                ..session.last
            });
        }
    }
}
