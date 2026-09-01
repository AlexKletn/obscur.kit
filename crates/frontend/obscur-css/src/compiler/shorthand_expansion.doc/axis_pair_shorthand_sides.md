Returns the two longhand [`PropertyId`]s (inline axis, block axis) that an
axis-pair shorthand expands into, or [`None`] for any other name. `overflow`
is the only one: css-overflow-3 §3 gives it the grammar `overflow-x
overflow-y`, so the *inline* side comes first and a single value sets both.
The longhands are the storage — nothing keeps a separate `overflow` value —
which is what lets `overflow: hidden; overflow-y: auto` cascade the way an
author expects.
