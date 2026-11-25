Returns whether a new entry of `incoming_bytes` may be admitted given the
cache's current size (spec §5.19 memory budgets).

This is `false` when the cache is disabled (spec §5.19 disable switch),
when adding `incoming_bytes` would push total bytes past `max_bytes`, or
when the additional entry would push the count past `max_entries`. A
saturating add keeps a very large `incoming_bytes` from overflowing.
