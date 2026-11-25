Resolves every unbound (or released) registration against `runtime`'s
current tree and registers it, returning how many were newly bound.

Idempotent: called after every commit, it does nothing at all once every
name has resolved and its node still answers to it. A name that resolves to
nothing — markup that is not on screen yet, or a `test-id` that was never
authored — stays pending and is retried on the next commit rather than
failing; a coordinate target never resolves to a node and therefore never
binds (see [`SelectorResolver`]).
