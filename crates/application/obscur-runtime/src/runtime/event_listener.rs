//! One listener attached to one node for one command, in one phase (spec §2.3).

use obscur_interaction::event::{EventPhase, ListenerPhase};

use super::CommandHandler;

/// A single listener registered on a node by
/// [`UiRuntime::add_event_listener`](super::UiRuntime::add_event_listener)
/// (spec §2.3).
///
/// A listener is *not* the same thing as a §2.6 command **handler**, and the
/// difference is worth stating because both are closures on a node:
///
/// - a **handler** ([`on_command_at`](super::UiRuntime::on_command_at)) is the one
///   piece of logic that *claims* a command; §2.6's chain offers the command to
///   each link in turn and stops at the first that has one. There is at most one
///   per `(node, command)`, and registering again replaces it.
/// - a **listener** *observes and steers* the same command as it propagates. Any
///   number may sit on one node, they run in registration order, and none of them
///   claims anything — they are §2.3's capture and bubble phases, which exist
///   precisely so a node that is not the target gets a say.
///
/// That is why listeners are an ordered `Vec` while handlers are a single slot:
/// `stop_immediate_propagation` — *"skip the remaining handlers on the current
/// node"* — has nothing to skip unless a node can carry more than one, so the
/// spec's own API implies the multiplicity.
pub(crate) struct EventListener {
    /// The phase this listener asked to run in.
    phase: ListenerPhase,
    /// The listener body, normalized like every other registered handler.
    handler: CommandHandler,
}

impl EventListener {
    /// Registers `handler` to run in `phase`.
    pub(crate) const fn new(phase: ListenerPhase, handler: CommandHandler) -> Self {
        Self { phase, handler }
    }

    /// Returns whether this listener runs during `phase` of a propagation.
    pub(crate) const fn runs_in(&self, phase: EventPhase) -> bool {
        self.phase.runs_in(phase)
    }

    /// Returns the listener body, for the walk to call.
    pub(crate) const fn handler_mut(&mut self) -> &mut CommandHandler {
        &mut self.handler
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use obscur_identity::NodeId;
    use obscur_interaction::event::{EventOrigin, UiEvent};

    use super::super::IntoCommandHandler;
    use super::*;

    #[test]
    fn a_capture_listener_runs_only_on_the_way_down_and_at_the_target() {
        let listener = EventListener::new(ListenerPhase::Capture, (|| {}).into_command_handler());
        assert!(listener.runs_in(EventPhase::Capture));
        assert!(listener.runs_in(EventPhase::Target));
        assert!(!listener.runs_in(EventPhase::Bubble));
    }

    #[test]
    fn a_bubble_listener_runs_only_on_the_way_up_and_at_the_target() {
        let listener = EventListener::new(ListenerPhase::Bubble, (|| {}).into_command_handler());
        assert!(!listener.runs_in(EventPhase::Capture));
        assert!(listener.runs_in(EventPhase::Target));
        assert!(listener.runs_in(EventPhase::Bubble));
    }

    #[test]
    fn handler_mut_runs_the_body() {
        let ran = Rc::new(Cell::new(false));
        let flag = Rc::clone(&ran);
        let mut listener = EventListener::new(
            ListenerPhase::Bubble,
            (move || flag.set(true)).into_command_handler(),
        );

        let mut event = UiEvent::new(EventOrigin::Pointer, NodeId::from_raw(2));
        listener.handler_mut().call(&mut event);

        assert!(ran.get());
    }
}
