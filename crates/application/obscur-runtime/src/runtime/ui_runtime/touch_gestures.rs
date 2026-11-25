use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::{EventOrigin, UiEvent};
use obscur_interaction::gesture::{GestureArena, GestureContender, RecognizerId, RecognizerState};
use obscur_interaction::pointer::{PointerEvent, PointerEventKind, PointerId};
use obscur_view::view::EVENT_TOUCH_GESTURE;

use crate::runtime::{CommandDispatch, PointerCompletion};

use super::UiRuntime;

const APP: RecognizerId = RecognizerId::from_raw(1);
const DEFAULT: RecognizerId = RecognizerId::from_raw(2);

#[derive(Debug)]
pub(super) struct TouchGesture {
    arena: GestureArena,
    pub(super) bound: NodeId,
    command: InternedSymbol,
    last: PointerEvent,
}

impl TouchGesture {
    fn new(bound: NodeId, command: InternedSymbol, last: PointerEvent) -> Self {
        let mut arena = GestureArena::new();
        arena.add(GestureContender::new(APP, bound));
        arena.add(GestureContender::new(DEFAULT, bound));
        Self {
            arena,
            bound,
            command,
            last,
        }
    }

    fn claim(&mut self, candidate: RecognizerId) -> bool {
        if let Some(winner) = self.arena.resolve().winner() {
            return candidate == winner;
        }
        self.arena.mark(candidate, RecognizerState::Recognized);
        self.arena.sweep().winner() == Some(candidate)
    }

    fn winner(&self) -> Option<RecognizerId> {
        self.arena.resolve().winner()
    }

    fn binding(&self) -> (NodeId, InternedSymbol) {
        (self.bound, self.command)
    }
}

impl UiRuntime {
    pub(super) fn update_touch_gesture(
        &mut self,
        event: PointerEvent,
        target: Option<NodeId>,
        press_changed: &mut bool,
    ) -> bool {
        if !event.device().is_touchscreen() || !event.is_trusted() {
            return false;
        }
        let pointer = event.pointer();
        if event.kind() == PointerEventKind::Down {
            if let Some(previous) = self.touch_gestures.remove(&pointer)
                && previous.winner() != Some(DEFAULT)
            {
                self.cancel_touch_candidate(previous);
            }
            if !self.pointer_bindings.binds(EVENT_TOUCH_GESTURE) {
                return false;
            }
            let Some((bound, command)) = target.and_then(|node| {
                CommandDispatch::binding_for(&self.tree, node, EVENT_TOUCH_GESTURE)
            }) else {
                return false;
            };
            let mut session = TouchGesture::new(bound, command, event);
            if let Some(winner) = self
                .touch_gestures
                .values()
                .filter(|other| other.binding() == session.binding())
                .find_map(TouchGesture::winner)
            {
                session.claim(winner);
            }
            self.touch_gestures.insert(pointer, session);
        }
        let Some(session) = self.touch_gestures.get_mut(&pointer) else {
            return false;
        };
        let terminal = matches!(
            event.kind(),
            PointerEventKind::Up | PointerEventKind::Cancel
        );
        let winner = session.winner();
        if winner == Some(DEFAULT) {
            if terminal {
                self.touch_gestures.remove(&pointer);
            }
            return false;
        }
        let owned = winner == Some(APP);
        let (bound, command) = session.binding();
        session.last = event;
        let claimed = self.tree.contains(bound)
            && self
                .dispatch_command_outcome(Some(bound), command, candidate_event(event))
                .default_prevented;
        if terminal {
            self.touch_gestures.remove(&pointer);
            return owned || claimed;
        }
        if claimed && !owned {
            // All active contacts sent to this binding form one candidate, so
            // a two-contact recognizer can claim both contacts at recognition.
            for (id, session) in &mut self.touch_gestures {
                if session.binding() == (bound, command) && session.claim(APP) {
                    *press_changed |= self
                        .pointers
                        .take_press(*id)
                        .and_then(|press| press.target)
                        .is_some();
                    self.pointers.end_pointer(*id, PointerCompletion::Cancelled);
                }
            }
        }
        owned || claimed
    }

    pub(super) fn default_won_touch_gesture(&mut self, pointer: PointerId) {
        let Some(session) = self.touch_gestures.get(&pointer) else {
            return;
        };
        if session.winner().is_some() {
            return;
        }
        let binding = session.binding();
        let cancelled: Vec<_> = self
            .touch_gestures
            .values_mut()
            .filter(|candidate| candidate.binding() == binding && candidate.winner().is_none())
            .map(|candidate| {
                candidate.claim(DEFAULT);
                candidate.last
            })
            .collect();
        for mut event in cancelled {
            event.kind = PointerEventKind::Cancel;
            self.dispatch_command_event(Some(binding.0), binding.1, candidate_event(event));
        }
    }

    fn cancel_touch_candidate(&mut self, mut session: TouchGesture) {
        session.last.kind = PointerEventKind::Cancel;
        if self.tree.contains(session.bound) {
            self.dispatch_command_event(
                Some(session.bound),
                session.command,
                candidate_event(session.last),
            );
        }
    }
}

fn candidate_event(pointer: PointerEvent) -> UiEvent {
    let mut event = UiEvent::from_pointer(pointer);
    event.origin = EventOrigin::Gesture;
    event
}
