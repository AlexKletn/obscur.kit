Dispatches `command` from `origin` as a full §2.3 propagation, carrying
`envelope`'s origin, trust, timestamp and device payload.

It runs the spec's four steps in order:

> Event propagation:
>
> 1. capture;
> 2. target;
> 3. bubble;
> 4. default action.

Steps 1–3 are [`propagate`](Self::propagate): the event descends the node
path to `origin`, fires at it, and ascends, running the registered
listeners and updating [`UiEvent::current_target`] at every step while
[`UiEvent::target`] stays put. Step 4 is [`route`](Self::route) plus
[`HandlerRegistry::dispatch_along`]: §2.6's chain, offering the command to
*focused node → focus scope → ancestors → window → application* and
stopping at the first link with a handler.

A command whose [`CommandState`](obscur_interaction::command::CommandState)
forbids dispatch is inert before any of this: no listener sees it either,
since a disabled command is not an event that happened.
