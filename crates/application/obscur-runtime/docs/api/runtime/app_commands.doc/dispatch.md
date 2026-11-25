Runs the fallback registered for `command` against the propagating
`event`, returning whether one ran.

Called by the runtime as the final link of the routing chain. Whatever the
handler sets on `event` — `prevent_default`, `stop_propagation` — is
visible to the caller afterwards, exactly as for the other four links.
