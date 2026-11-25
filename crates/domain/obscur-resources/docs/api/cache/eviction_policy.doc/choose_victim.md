Chooses the entry to evict from `entries`, or `None` if it is empty (spec
§5.19 eviction).

The choice is deterministic: each policy's primary key is compared first,
and every tie is broken by successively finer keys down to the entry's
[`CacheKey`](crate::cache::CacheKey), so the outcome never depends on the
order of `entries`.
