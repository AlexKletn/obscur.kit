Records `runs` for `text` under these style inputs, rotating the
generations first if the young one is full.

A rotation drops the old generation and nothing else, so the entries this
insertion displaces are exactly those not looked at since the previous
rotation — never the whole table.
