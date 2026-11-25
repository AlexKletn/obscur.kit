Returns `true` when this kind isolates its scope's non-inherited
properties at the boundary.

Isolated for [`ScopeKind::Component`] and [`ScopeKind::Module`]; not
isolated for [`ScopeKind::Global`] and [`ScopeKind::Open`], which are
deliberately open (§4.15).
