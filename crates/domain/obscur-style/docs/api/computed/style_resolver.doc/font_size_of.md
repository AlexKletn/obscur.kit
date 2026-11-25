Reads a node's computed `font-size` in logical pixels.

Every style [`resolve`](Self::resolve) produces carries an **absolute**
`font-size` (the resolver folds it before anything else), so this never
falls back for a resolved style. The [`INITIAL_FONT_SIZE`] default covers
only a hand-built [`ComputedStyle`] that never went through the resolver.
