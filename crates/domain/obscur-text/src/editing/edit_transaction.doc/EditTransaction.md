An atomic, invertible record of one text replacement for local undo/redo
(spec §5.11 editing engine — local undo/redo primitives).

A transaction captures a single splice: the byte [`range`](Self::range) in
the *pre-edit* text that was replaced, the [`removed`](Self::removed) text
that occupied it, and the [`inserted`](Self::inserted) text put in its place,
plus the selection immediately before and after. Because it stores both sides
of the splice it is fully invertible ([`invert`](Self::invert)) with no access
to the buffer, which is the basis of the undo stack: undo applies the
inverse, redo re-applies the original.

Pure insertion is `removed == ""` with an empty `range`; pure deletion is
`inserted == ""`.
