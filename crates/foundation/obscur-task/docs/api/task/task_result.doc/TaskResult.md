The typed message a finished task delivers (spec §3.3: "results return as
typed messages").

# Typed, not erased

`T` is the caller's own payload type — a decoded resource, a parsed document,
whatever the task produced. The envelope adds only what the *runtime* needs to
route it: which task it came from, and what that task was scoped to. Nothing
here is `Any`-erased or stringly-typed, so a receiver matches on a real Rust
value rather than re-parsing a message.

# It carries no generation

Deliberately. §3.3 also requires that "stale work is discarded using
generation/version IDs", but the ID that decides staleness belongs to the
*domain the work targets* — a resource load is stale against its
`ResourceVersion`, which the resource hub already checks on completion.
Putting a second, generic generation here would create a competing answer to
"is this stale", and the two could disagree. The payload carries its own.
