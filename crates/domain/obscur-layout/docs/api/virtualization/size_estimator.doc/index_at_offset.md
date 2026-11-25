The index of the item whose extent band contains `offset` (spec §4.9).

Bands are half-open: the item at `index` spans
`[offset_of(index), offset_of(index + 1))`. An `offset` at or below `0`
resolves to item `0`; an `offset` at or beyond the total extent resolves to
`item_count` (one past the last item). When `item_count` is `0` the result
is `0`.
