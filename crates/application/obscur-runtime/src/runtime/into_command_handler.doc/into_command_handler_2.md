Wraps a zero-argument handler in a closure that discards the event.

The discard happens once, here, at registration — not per dispatch — so the
convenience form is free at call time.
