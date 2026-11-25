//! The one stored shape every registered handler is normalized to.

use obscur_interaction::event::UiEvent;

#[doc = include_str!("../../docs/api/runtime/command_handler.doc/CommandHandler.md")]
pub struct CommandHandler {
    handler: Box<dyn FnMut(&mut UiEvent)>,
}

impl CommandHandler {
    /// Wraps a boxed event-taking closure.
    #[must_use]
    pub fn new(handler: Box<dyn FnMut(&mut UiEvent)>) -> Self {
        Self { handler }
    }

    #[doc = include_str!("../../docs/api/runtime/command_handler.doc/call.md")]
    pub fn call(&mut self, event: &mut UiEvent) {
        (self.handler)(event);
    }
}

impl std::fmt::Debug for CommandHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CommandHandler(..)")
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use obscur_identity::NodeId;
    use obscur_interaction::event::EventOrigin;

    use super::*;

    #[test]
    fn call_runs_the_closure_with_the_event() {
        let seen = Rc::new(Cell::new(None));
        let sink = Rc::clone(&seen);
        let mut handler = CommandHandler::new(Box::new(move |event: &mut UiEvent| {
            sink.set(event.target());
        }));

        let mut event = UiEvent::new(EventOrigin::Pointer, NodeId::from_raw(9));
        handler.call(&mut event);

        assert_eq!(seen.get(), Some(NodeId::from_raw(9)));
    }

    /// The flags a handler sets are the caller's to read — otherwise
    /// `stop_propagation` would be a no-op with a name.
    #[test]
    fn what_the_handler_sets_is_visible_to_the_caller() {
        let mut handler = CommandHandler::new(Box::new(|event: &mut UiEvent| {
            event.stop_propagation();
            event.prevent_default();
        }));

        let mut event = UiEvent::new(EventOrigin::Keyboard, NodeId::from_raw(1));
        handler.call(&mut event);

        assert!(event.is_stopped());
        assert!(event.is_default_prevented());
    }
}
