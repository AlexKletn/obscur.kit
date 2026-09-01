The properties whose value is a keyword.

A keyword reaches this judgement already lowered by the grammar the property
named, so the closed-set families arrive as the same reserved symbols
(`layout_keyword`) a keyword written directly on the property lowers to:
`--o: hidden; overflow: var(--o)` clips exactly as `overflow: hidden` does,
and a word the property does not have — `--o: sideways` — is no value at all
by then, rather than a symbol invented for it.

What is left over is the open-set keywords, `gap: normal` and its like: those
still carry a symbol interned per stylesheet, so two sheets spell the same
word differently. That is a property of every per-sheet interner here and not
of this judgement, and it is why the answer is by property rather than by
symbol.
