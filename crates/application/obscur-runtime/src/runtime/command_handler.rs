//! The one stored shape every registered handler is normalized to.

use obscur_interaction::event::UiEvent;

/// A registered command handler, after normalization: one boxed closure that
/// takes the propagating [`UiEvent`] (spec §2.3).
///
/// # Why there is exactly one stored shape
///
/// An application may register a handler in two shapes — `FnMut()` when it needs
/// nothing from the event, `FnMut(&mut UiEvent)` when it does — and
/// [`IntoCommandHandler`](super::IntoCommandHandler) converts both into this one.
/// The runtime's handler tables therefore stay homogeneous
/// (`HashMap<_, CommandHandler>`), and the dispatch walk has one call shape to
/// honour rather than a match over arities at every one of §2.6's five links. The
/// zero-argument form is wrapped in a closure that ignores the event, so it costs
/// nothing at dispatch time and cannot observe the difference.
///
/// # Why `&mut UiEvent` and not `&UiEvent`
///
/// Because §2.3's API is *mutating*: `stop_propagation`,
/// `stop_immediate_propagation` and `prevent_default` all change the event's
/// state, and they are the entire point of handing the event to a handler. A
/// `&UiEvent` handler could read the payload but could not steer anything, so the
/// three flags §2.3 names would be exported and still unreachable — the exact
/// shape of the gap this type closes. The alternative (interior mutability behind
/// `&UiEvent`, as the DOM does) would buy nothing here: a handler runs alone,
/// under `&mut self` on the runtime, so there is no aliasing to work around.
pub struct CommandHandler {
    handler: Box<dyn FnMut(&mut UiEvent)>,
}

impl CommandHandler {
    /// Wraps a boxed event-taking closure.
    #[must_use]
    pub fn new(handler: Box<dyn FnMut(&mut UiEvent)>) -> Self {
        Self { handler }
    }

    /// Runs the handler against the propagating `event`.
    ///
    /// The event is passed by mutable reference so anything the handler sets —
    /// [`UiEvent::stop_propagation`], [`UiEvent::prevent_default`] — is visible to
    /// the walk that called it, which is what makes those methods do something.
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
