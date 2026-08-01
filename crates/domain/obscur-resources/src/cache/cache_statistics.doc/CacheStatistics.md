Running metrics for one runtime cache (spec §5.19 *`DevTools` metrics; correctness
transparency*).

Counts hits, misses, evictions, and insertions, and tracks the live entry
count and total bytes. These figures surface a cache's behaviour to `DevTools`
and make its correctness transparent — for example, an unexpectedly low
[`hit_rate`](CacheStatistics::hit_rate) or a runaway eviction count is visible
rather than hidden.

This is pure bookkeeping: every counter uses saturating arithmetic, so it can
never overflow or underflow, and it holds no artifact data. All fields are
integers, so the type derives `Eq`, `Hash`, and `Default`.
