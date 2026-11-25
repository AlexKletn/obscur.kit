Activates the control at `node`: dispatches its
[`CommandDispatch::activation_command_for`] command
(`EVENT_CLICK` else `EVENT_CHANGE`) as a §2.3 propagation carrying
`envelope`, running the registered handler. Returns whether a handler
actually ran.

This is the shared entry point for both a pointer click (in
[`handle_pointer`](Self::handle_pointer)) and an Enter/Space keypress on
the focused control (in [`handle_key`](Self::handle_key)). With no
activation binding in the node's ancestor chain — or no handler
registered for the resolved command — it is a harmless no-op that returns
`false` and never panics.

`envelope` is the *device* event that caused the activation, built by
[`UiEvent::from_pointer`] or [`UiEvent::from_key`] at the call site. It is
what carries §2.4's payload — the pointer's id, position, button, device
and pressure, or the key — through to the handler. Before it, activation
dispatched a command symbol and nothing else, so a handler could not tell
a click from an Enter, let alone read where the click landed.
