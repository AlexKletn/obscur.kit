The visual highlight of a [`TextSelection`](super::TextSelection) as a set of
rectangles (spec §5.11 editing engine).

A selection that spans several wrapped lines is drawn as one partial
rectangle per covered line — the leading line from the start offset to its
trailing edge, whole intermediate lines, and the trailing line from its start
to the end offset. Each rectangle is derived from the layout's per-line caret
geometry (see the `caret_rect` idiom on the layout hit-tester); this type is
a pure container that stores those rectangles and answers spatial queries
over them. It performs no layout of its own.
