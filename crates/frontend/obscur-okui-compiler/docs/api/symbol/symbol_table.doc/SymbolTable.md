An arena of lexical [`Scope`]s with a current-scope cursor (spec §2.10).

Scopes are stored contiguously in `scopes`; index `0` is always the root.
Each non-root scope records the arena index of its parent, so the scope tree
is encoded by those parent links. The `current` cursor names the scope that
[`define`](Self::define) writes into and that [`lookup`](Self::lookup) starts
from.

Lexical resolution walks outward: [`lookup`](Self::lookup) searches the
current scope, then its parent, and so on to the root, returning the first
match. This lets an inner scope (for example a `<For>` body) shadow and
extend the bindings of its enclosing scopes.
