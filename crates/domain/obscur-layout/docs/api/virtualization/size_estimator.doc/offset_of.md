The cumulative extent of every item before `index`, i.e. the scroll offset
at which the item at `index` begins (spec §4.9).

`index` is clamped to `item_count`, so `offset_of(item_count, ..)` is the
total extent.
