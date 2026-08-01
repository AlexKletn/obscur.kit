A local identifier isolated to a style scope.

CSS-module isolation (§4.15) means a component's internal class names do not
need global uniqueness: two different modules may both use `.item` without
colliding. [`ScopedIdent`] models that by pairing a local
[`InternedSymbol`] with the [`ScopeId`] it belongs to, so identity is the
*combination* of scope and local name rather than the name alone.
