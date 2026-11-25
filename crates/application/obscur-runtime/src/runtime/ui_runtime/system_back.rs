use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::{EventOrigin, EventPayload, UiEvent};
use obscur_interaction::gesture::{GesturePhase, SystemBackEvent};
use obscur_view::view::EVENT_SYSTEM_BACK;

use crate::runtime::CommandDispatch;

use super::UiRuntime;

#[derive(Debug, Clone, Copy)]
pub(super) struct SystemBackSession {
    pub(super) bound: NodeId,
    command: InternedSymbol,
    progress: f32,
}

impl UiRuntime {
    /// Receives predictive Back from a trusted window host. On start, the host supplies the focused surface or node. The same binding receives the rest of the sequence. The host must forward cancellation on focus loss and cancel any raw contact the system claimed.
    pub fn handle_system_back(
        &mut self,
        target: Option<NodeId>,
        mut event: SystemBackEvent,
    ) -> bool {
        if event.phase == GesturePhase::Started {
            if let Some(previous) = self.system_back.take() {
                self.dispatch_system_back(previous, GesturePhase::Cancelled);
            }
            let Some((bound, command)) = target
                .and_then(|node| CommandDispatch::binding_for(&self.tree, node, EVENT_SYSTEM_BACK))
            else {
                return false;
            };
            self.system_back = Some(SystemBackSession {
                bound,
                command,
                progress: 0.0,
            });
        }
        let Some(mut session) = self.system_back.take() else {
            return false;
        };
        if !self.tree.contains(session.bound) {
            return false;
        }
        if event.progress.is_finite() {
            session.progress = event.progress.clamp(0.0, 1.0);
        } else if event.phase != GesturePhase::Cancelled {
            event.phase = GesturePhase::Cancelled;
        }
        if matches!(event.phase, GesturePhase::Started | GesturePhase::Updated) {
            self.system_back = Some(session);
        }
        self.dispatch_system_back(session, event.phase)
    }

    fn dispatch_system_back(&mut self, session: SystemBackSession, phase: GesturePhase) -> bool {
        if !self.tree.contains(session.bound) {
            return false;
        }
        let mut event = UiEvent::new(EventOrigin::Gesture, session.bound).as_trusted();
        event.payload = EventPayload::SystemBack(SystemBackEvent {
            phase,
            progress: session.progress,
        });
        self.dispatch_command_event(Some(session.bound), session.command, event)
    }
}
