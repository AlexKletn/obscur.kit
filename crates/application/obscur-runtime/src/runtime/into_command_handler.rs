//! The marker-generic conversion that lets a handler take the event, or nothing.

use obscur_interaction::event::UiEvent;

use super::{CommandHandler, WithEvent, WithoutEvent};

#[doc = include_str!("../../docs/api/runtime/into_command_handler.doc/IntoCommandHandler.md")]
pub trait IntoCommandHandler<M> {
    /// Normalizes this handler into the runtime's stored shape.
    fn into_command_handler(self) -> CommandHandler;
}

impl<F> IntoCommandHandler<WithoutEvent> for F
where
    F: FnMut() + 'static,
{
    #[doc = include_str!("../../docs/api/runtime/into_command_handler.doc/into_command_handler_2.md")]
    fn into_command_handler(mut self) -> CommandHandler {
        CommandHandler::new(Box::new(move |_event: &mut UiEvent| self()))
    }
}

impl<F> IntoCommandHandler<WithEvent> for F
where
    F: FnMut(&mut UiEvent) + 'static,
{
    /// Stores an event-taking handler as-is.
    fn into_command_handler(self) -> CommandHandler {
        CommandHandler::new(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use obscur_identity::NodeId;
    use obscur_interaction::event::EventOrigin;

    use super::*;

    fn event() -> UiEvent {
        UiEvent::new(EventOrigin::Pointer, NodeId::from_raw(4))
    }

    /// The convenience form: a handler that wants nothing still registers, and
    /// still runs.
    #[test]
    fn a_zero_argument_closure_converts_and_runs() {
        let ran = Rc::new(Cell::new(0_u32));
        let counter = Rc::clone(&ran);
        let mut handler = (move || counter.set(counter.get() + 1)).into_command_handler();

        handler.call(&mut event());
        handler.call(&mut event());

        assert_eq!(ran.get(), 2);
    }

    /// The widened form: the same registration point, and the event arrives.
    #[test]
    fn an_event_taking_closure_receives_the_event() {
        let seen = Rc::new(Cell::new(None));
        let sink = Rc::clone(&seen);
        let mut handler = (move |e: &mut UiEvent| sink.set(e.target())).into_command_handler();

        handler.call(&mut event());

        assert_eq!(seen.get(), Some(NodeId::from_raw(4)));
    }

    /// Both shapes end up as the same stored type — which is the property that
    /// keeps the dispatch walk free of arity matching.
    #[test]
    fn both_shapes_normalize_to_one_stored_type() {
        let handlers: Vec<CommandHandler> = vec![
            (|| {}).into_command_handler(),
            (|_e: &mut UiEvent| {}).into_command_handler(),
        ];
        assert_eq!(handlers.len(), 2);
    }
}
