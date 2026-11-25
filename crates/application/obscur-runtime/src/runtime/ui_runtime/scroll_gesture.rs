use obscur_geometry::Point;
use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::{EventOrigin, UiEvent};
use obscur_interaction::gesture::{GestureArena, GestureContender, RecognizerId, RecognizerState};
use obscur_interaction::pointer::{WheelDeltaMode, WheelEvent, WheelPhase};
use obscur_view::view::EVENT_SCROLL_GESTURE;

use crate::frame::FrameReason;
use crate::runtime::ScrollTargets;

use super::UiRuntime;

const APP: RecognizerId = RecognizerId::from_raw(1);
const SCROLL: RecognizerId = RecognizerId::from_raw(2);
const SCROLL_SLOP: f32 = 8.0;

#[derive(Debug)]
pub(super) struct ScrollGesture {
    arena: GestureArena,
    binding: (NodeId, InternedSymbol, Point),
    scroller: Option<NodeId>,
    origin: Point,
    last: WheelEvent,
    pending: Point,
}

impl ScrollGesture {
    fn new(binding: (NodeId, InternedSymbol, Point), origin: Point, wheel: WheelEvent) -> Self {
        let mut arena = GestureArena::new();
        arena.add(GestureContender::new(APP, binding.0));
        Self {
            arena,
            binding,
            scroller: None,
            origin,
            last: wheel,
            pending: Point::new(0.0, 0.0),
        }
    }

    fn claim(&mut self, recognizer: RecognizerId) {
        self.arena.mark(recognizer, RecognizerState::Recognized);
        self.arena.sweep();
    }
}

impl UiRuntime {
    pub(super) fn arbitrate_scroll_candidate(
        &mut self,
        wheel: WheelEvent,
        content: Point,
    ) -> Option<NodeId> {
        if wheel.mode != WheelDeltaMode::Pixels || !wheel.trust.is_trusted() {
            return None;
        }
        if matches!(wheel.phase, WheelPhase::Unspecified | WheelPhase::Started) {
            self.interrupt_scroll_candidate();
            self.scroll_gesture_blocked = false;
        }
        let terminal = matches!(wheel.phase, WheelPhase::Ended | WheelPhase::Cancelled);
        if self.scroll_gesture_blocked {
            self.scroll_gesture_blocked = !terminal;
            return None;
        }
        if self.scroll_gesture.is_none()
            && matches!(wheel.phase, WheelPhase::Started | WheelPhase::Moved)
            && self.pointer_bindings.binds(EVENT_SCROLL_GESTURE)
        {
            let Some(binding) = self.point_event_binding(content, EVENT_SCROLL_GESTURE) else {
                self.scroll_gesture_blocked = true;
                return None;
            };
            self.scroll_gesture = Some(ScrollGesture::new(binding, content, wheel));
        }
        let mut session = self.scroll_gesture.take()?;
        let winner = session.arena.resolve().winner();
        let binding = session.binding.0;
        if wheel.phase != WheelPhase::Cancelled
            && (!wheel.delta_x.is_finite() || !wheel.delta_y.is_finite())
        {
            self.cancel_scroll_candidate(&session);
            self.scroll_gesture_blocked = !terminal;
            return Some(binding);
        }

        let result = if winner == Some(SCROLL) {
            if wheel.phase != WheelPhase::Cancelled {
                self.scroll_gesture_by(session.scroller, Point::new(wheel.delta_x, wheel.delta_y));
            }
            session.scroller.unwrap_or(binding)
        } else {
            session.last = wheel;
            let claimed = self.dispatch_scroll_candidate(&session, wheel);
            if winner == Some(APP) || claimed {
                if winner.is_none() {
                    session.claim(APP);
                }
                binding
            } else if wheel.phase != WheelPhase::Cancelled {
                session.pending.x += wheel.delta_x;
                session.pending.y += wheel.delta_y;
                let vertical = session.pending.y.abs();
                let horizontal = session.pending.x.abs();
                let direction = if horizontal > vertical {
                    Point::new(session.pending.x, 0.0)
                } else {
                    Point::new(0.0, session.pending.y)
                };
                if terminal || vertical.max(horizontal) >= SCROLL_SLOP * self.surface.device_scale()
                {
                    session.scroller =
                        ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
                            .scrollable_at(session.origin, direction);
                }
                if let Some(node) = session.scroller.filter(|_| !terminal) {
                    self.cancel_scroll_candidate(&session);
                    session.arena.add(GestureContender::new(SCROLL, node));
                    session.claim(SCROLL);
                }
                if terminal || session.arena.resolve().winner() == Some(SCROLL) {
                    self.scroll_gesture_by(session.scroller, session.pending);
                    session.scroller.unwrap_or(binding)
                } else {
                    binding
                }
            } else {
                binding
            }
        };
        if !terminal && self.tree.contains(binding) {
            self.scroll_gesture = Some(session);
        }
        Some(result)
    }

    pub(super) fn refuse_scroll_candidate(&mut self, wheel: WheelEvent) {
        if wheel.trust.is_trusted()
            && wheel.mode == WheelDeltaMode::Pixels
            && wheel.phase != WheelPhase::Unspecified
        {
            self.suspend_scroll_candidate();
            self.scroll_gesture_blocked =
                matches!(wheel.phase, WheelPhase::Started | WheelPhase::Moved);
        }
    }

    pub(super) fn suspend_scroll_candidate(&mut self) {
        self.scroll_gesture_blocked |= self.scroll_gesture.is_some();
        self.interrupt_scroll_candidate();
    }

    fn interrupt_scroll_candidate(&mut self) {
        if let Some(session) = self.scroll_gesture.take() {
            self.cancel_scroll_candidate(&session);
        }
    }

    pub(super) fn retain_live_scroll_candidate(&mut self) {
        if self
            .scroll_gesture
            .as_ref()
            .is_some_and(|session| !self.tree.contains(session.binding.0))
        {
            self.scroll_gesture = None;
            self.scroll_gesture_blocked = true;
        }
    }

    fn scroll_gesture_by(&mut self, node: Option<NodeId>, delta: Point) {
        let scale = self.surface.camera_scale();
        let delta = Point::new(delta.x / scale, delta.y / scale);
        if let Some(node) = node.filter(|node| self.tree.contains(*node))
            && delta != Point::ORIGIN
            && self.scroll_chain_from(node, delta)
        {
            self.request_frame(FrameReason::Input);
        }
    }

    fn cancel_scroll_candidate(&mut self, session: &ScrollGesture) {
        if session.arena.resolve().winner() != Some(SCROLL) {
            self.dispatch_scroll_candidate(session, session.last.with_phase(WheelPhase::Cancelled));
        }
    }

    fn dispatch_scroll_candidate(&mut self, session: &ScrollGesture, wheel: WheelEvent) -> bool {
        let (node, command, anchor) = session.binding;
        if !self.tree.contains(node) {
            return false;
        }
        let mut event = UiEvent::from_wheel(wheel.at(anchor));
        event.origin = EventOrigin::Gesture;
        self.dispatch_command_outcome(Some(node), command, event)
            .default_prevented
    }
}
