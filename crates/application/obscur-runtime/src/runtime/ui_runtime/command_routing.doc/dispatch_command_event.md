Dispatches `command` from `origin` as a full §2.3 propagation, carrying
`envelope`'s origin, trust, timestamp and device payload. Returns whether a
§2.6 link claimed the command.

This is the one place the §2.3 event model and §2.6's routing chain meet,
and it runs the spec's four steps in order:

> Event propagation:
>
> 1. capture;
> 2. target;
> 3. bubble;
> 4. default action.

Steps 1–3 are [`CommandDispatch`]'s listener walk: the event descends the node
path to `origin`, fires at it, and ascends, running the listeners
registered with [`add_event_listener`](Self::add_event_listener) and
updating [`UiEvent::current_target`] at every step while
[`UiEvent::target`] stays put. Step 4 is
[`route_command`](Self::route_command): §2.6's chain, offering the command
to *focused node → focus scope → ancestors → window → application* and
stopping at the first link with a handler.

# Why the §2.6 chain is step 4

Because running the node's bound handler is what a click's default action
*is* — the same reading [`handle_pointer`](Self::handle_pointer) already
takes where it gates activation on trust ("§2.3 propagation step 4, the
**default action**"). Two consequences follow, and both are observable:

- a listener calling [`UiEvent::prevent_default`] suppresses the command's
  handler entirely, while leaving the rest of the listener walk running;
- a listener calling [`UiEvent::stop_propagation`] halts the walk *and*
  the chain, because the chain is the remainder of the propagation rather
  than a separate mechanism bolted beside it.

# Gates and frames

A command whose [`CommandState`] forbids dispatch is inert before any of
this: no listener sees it either, since a disabled command is not an event
that happened. A frame is requested when anything ran — a listener or a
handler — because either may have mutated application state.
