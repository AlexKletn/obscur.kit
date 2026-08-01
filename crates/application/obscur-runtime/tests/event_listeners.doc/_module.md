§2.3's event model, driven end to end: capture, target, bubble, default action —
and the event object a handler is actually given.

Twelve tests, and the reason they are one file is that every one of them was
*unreachable* before the runtime drove this model. A handler used to be an
`impl FnMut()` given nothing at all, and `UiEvent`/`EventDispatcher` were called
by no code in the workspace, so none of the questions below could even be asked.

The payload half. One command symbol bound by two sibling rows: a single handler
learns which one the user hit, from `e.target()`, where the only previous answer
was a bespoke signature carved out for the one case that forced it. The pointer
payload §2.4 enumerates — id, device, buttons, position, delta, pressure,
primary status, hover capability — reaches the handler intact; so does the trust
bit, and so does the *key* that caused an Enter activation. A programmatic
`dispatch_command` has no device behind it and says so: synthetic origin, empty
payload, and no trust conferred by omission.

The propagation half. The phases run strictly outermost-first, then the target's
own listeners in registration order, then strictly innermost-first, with
`current_target` moving at every step and `target` never moving — the property
that makes them two values. `stop_propagation` ends the walk *and* the default
action, because §2.6's chain is the outward leg of the same propagation;
`stop_immediate_propagation` additionally skips the remaining listeners on the
same node, which is the only thing that distinguishes the two;
`prevent_default` suppresses the bound command's handler while leaving the walk
intact, which is the only thing that distinguishes it from a stop. A capture
listener on an ancestor can take the interaction away from the descendant it was
aimed at — what the capture phase is *for*. The zero-argument handler shape
still registers and still runs alongside the event-taking one. And
`clear_node_commands` reaps a node's listeners with its handler, or a churning
subtree would leak one closure per rebuilt row.
