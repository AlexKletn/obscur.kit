#![doc = include_str!("event_listener.doc/_module.md")]

use obscur_interaction::event::{EventPhase, ListenerPhase};

use super::CommandHandler;

#[doc = include_str!("event_listener.doc/EventListener.md")]
pub(crate) struct EventListener {
    #[doc = include_str!("event_listener.doc/phase.md")]
    phase: ListenerPhase,
    #[doc = include_str!("event_listener.doc/handler.md")]
    handler: CommandHandler,
}

impl EventListener {
    #[doc = include_str!("event_listener.doc/new.md")]
    pub(crate) const fn new(phase: ListenerPhase, handler: CommandHandler) -> Self {
        Self { phase, handler }
    }

    #[doc = include_str!("event_listener.doc/runs_in.md")]
    pub(crate) const fn runs_in(&self, phase: EventPhase) -> bool {
        self.phase.runs_in(phase)
    }

    #[doc = include_str!("event_listener.doc/handler_mut.md")]
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
