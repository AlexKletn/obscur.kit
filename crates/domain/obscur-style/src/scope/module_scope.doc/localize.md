Maps a local identifier into this scope, producing a [`ScopedIdent`].

This is how module isolation avoids requiring global uniqueness: the same
`local` mapped through two different [`ModuleScope`]s yields two distinct
[`ScopedIdent`]s that never collide (§4.15).
