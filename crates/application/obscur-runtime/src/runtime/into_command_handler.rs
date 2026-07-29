//! The marker-generic conversion that lets a handler take the event, or nothing.

use obscur_interaction::event::UiEvent;

use super::CommandHandler;

/// The marker for a handler that takes **no** arguments: `FnMut()`.
///
/// It has no fields and no behaviour; it exists only so the two blanket impls of
/// [`IntoCommandHandler`] below are impls of *different* traits
/// (`IntoCommandHandler<WithoutEvent>` and `IntoCommandHandler<WithEvent>`) and so
/// cannot overlap. It lives beside the trait it tags rather than in its own file
/// because it is part of that trait's contract and meaningless apart from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithoutEvent;

/// The marker for a handler that takes the propagating event:
/// `FnMut(&mut UiEvent)`.
///
/// See [`WithoutEvent`] for why the markers exist at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithEvent;

/// Converts either accepted handler shape into the one stored
/// [`CommandHandler`] (spec §2.3).
///
/// # What this trait is for
///
/// §2.3 requires that a handler be able to read `target` / `current_target`, the
/// device payload and the trust flag, and to call `stop_propagation`,
/// `stop_immediate_propagation` and `prevent_default`. All of that needs the
/// event, so the handler signature has to widen to `FnMut(&mut UiEvent)`. But the
/// zero-argument form is the *right* shape for the majority of handlers — an
/// increment, a navigation, a quit — and forcing every one of them to accept and
/// ignore a parameter would be a tax on the common case. So both are accepted:
///
/// ```ignore
/// rt.on_command(SAVE, || save());                                  // WithoutEvent
/// rt.on_command(PICK, |e: &mut UiEvent| pick(e.target()));         // WithEvent
/// ```
///
/// # Why a marker generic, and not one trait
///
/// A single trait cannot cover both signatures. The natural attempt —
///
/// ```text
/// impl<F: FnMut()> IntoCommandHandler for F { … }
/// impl<F: FnMut(&mut UiEvent)> IntoCommandHandler for F { … }
/// ```
///
/// — is rejected as a conflicting implementation: coherence compares the *self*
/// types, and both impls have the self type `F`, so the compiler must assume some
/// future type implements both `FnMut()` and `FnMut(&mut UiEvent)` (a hand-written
/// type trivially can) and refuses the pair. Adding the marker type parameter
/// makes them impls of two distinct traits, which is legal, while the call site
/// stays a single method: `on_command<M, H: IntoCommandHandler<M>>` lets inference
/// pick `M` from the closure's arity, because only one of the two candidate impls
/// has its `FnMut` bound satisfied.
///
/// # The one cost, stated plainly
///
/// Inference needs the closure's parameter type to be *known*, which for a
/// one-argument closure means annotating it: `|e: &mut UiEvent| …` rather than
/// `|e| …`. An unannotated `|e|` leaves both `M` and the closure's own signature
/// open at once and the compiler cannot resolve either from the other. The
/// zero-argument form needs no annotation.
pub trait IntoCommandHandler<M> {
    /// Normalizes this handler into the runtime's stored shape.
    fn into_command_handler(self) -> CommandHandler;
}

impl<F> IntoCommandHandler<WithoutEvent> for F
where
    F: FnMut() + 'static,
{
    /// Wraps a zero-argument handler in a closure that discards the event.
    ///
    /// The discard happens once, here, at registration — not per dispatch — so the
    /// convenience form is free at call time.
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
