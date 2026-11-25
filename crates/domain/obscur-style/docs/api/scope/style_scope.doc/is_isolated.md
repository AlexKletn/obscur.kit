Returns `true` when this scope isolates non-inherited properties at its
boundary (delegates to [`ScopeKind::is_isolated`]).

`Global` and `Open` scopes are not isolated; `Component` and `Module`
scopes are (§4.15).
