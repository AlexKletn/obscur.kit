Runs the listeners on `node` for `command` that apply in `phase`, in
registration order, returning how many ran.

Honors [`UiEvent::stop_immediate_propagation`] *within* the node: the
remaining listeners on this same node are skipped, which is the one thing
distinguishing it from the ordinary stop. A listener that does not apply to
this phase is skipped without counting — it is not this event's listener.

Holding a mutable borrow of the listener table across the call is sound for
the same reason [`AppCommands::dispatch`] documents: a handler is a closure
over application state with no handle to the runtime, so it cannot re-enter
this table.
