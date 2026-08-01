A structured, typed summary of an inspected target.

This crate cannot serialize the live runtime, so the payload is a typed
container the server fills from its runtime bridge: a human-readable
`description` plus coarse counts and ids. It is deliberately typed and
structured rather than free-form log text (spec §7.8).
