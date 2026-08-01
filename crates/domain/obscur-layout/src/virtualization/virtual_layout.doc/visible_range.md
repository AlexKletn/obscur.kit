The materialized range for a viewport of `viewport_extent` logical pixels
starting at `scroll_offset`, over `item_count` items (spec §4.9).

The visible band runs from the item at `scroll_offset` through the item at
`scroll_offset + viewport_extent`, i.e. `[first, last + 1)`. That band is
then widened by the overscan policy and clamped to `[0, item_count]`. An
empty collection yields an empty range.
