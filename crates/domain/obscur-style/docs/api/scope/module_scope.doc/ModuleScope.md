CSS-module isolation for a single scope root (§4.15).

It carries only the [`ScopeId`] it localizes into; the actual selector
rewriting lives in the matcher / `obscur-css`. `Copy + Eq + Hash`.
