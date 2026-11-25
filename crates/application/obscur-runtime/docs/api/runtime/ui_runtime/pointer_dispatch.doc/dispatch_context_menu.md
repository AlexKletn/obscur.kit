Dispatches `contextmenu` for a secondary-button press and reports whether a
listener refused it.

Returns `false` for every other event — a primary press, a middle press, a
release, a move — so the whole cost of this on the ordinary path is one
comparison of a `kind` and a `button` the caller already holds.

Deliberately **not** behind `PointerEventBindings`, unlike the raw pointer names
and the crossings. That gate exists for the pointer's report rate: a move
arrives hundreds of times a second and must not pay an ancestor walk it will
throw away. A secondary press arrives at the rate a hand can click, so the walk
below is once per right click — and an eighth bit spent on it would be the bit
the next reserved name has to grow the word for.
