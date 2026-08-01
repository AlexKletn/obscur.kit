Resolves an attribute name to the [`InternedSymbol`] it binds to, with no tag
context.

The reserved CSS/state attributes map to the `obscur-view` symbols the
reconciler and matcher read; any other name is hashed into the high symbol
namespace (see [`fallback_symbol`]).

Prefer [`attr_symbol_on`] when the tag is known: one binding (`name` on a
`<slot>`) depends on it, and this function cannot see it.
