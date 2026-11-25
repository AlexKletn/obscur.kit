What an `auto` margin does to a grid item inside its grid area
(`specs/w3c/css-grid-2.txt:3323`).

A *length* margin needs nothing of its own: it is subtracted from the grid area
before the item is sized, and [`MarginEdges::resolve`](crate::model::MarginEdges::resolve)
already answers `0` for `auto`, so the reservation arithmetic in
[`GridLayout`](super::GridLayout) reads the same for both kinds.

`auto` is the case that differs, and it differs in two places at once, which is
why this is an entity rather than one expression at the point of use. It
suppresses the stretch — `specs/w3c/css-grid-2.txt:3328`: "if `justify-self` or
`align-self` compute to a value other than `stretch` **or margins are auto**,
grid items will auto-size to fit their contents" — and it then takes the free
space that leaves, in place of the alignment that would otherwise have had it.
[`absorbs_free_space`] is the first question, [`auto_lead`] the second, and they
are asked at opposite ends of the pass.
