The strategy a cache uses to choose an eviction victim (spec §5.19 eviction).

Every policy is **deterministic**: given the same slice of entries it always
selects the same victim, breaking every tie down to the entry's
[`CacheKey`](crate::cache::CacheKey) so the result never depends on slice
order alone. This keeps eviction reproducible for tests and snapshots.

- [`Lru`](EvictionPolicy::Lru) — evict the least recently used entry.
- [`Lfu`](EvictionPolicy::Lfu) — evict the least frequently used entry.
- [`VisibilityAware`](EvictionPolicy::VisibilityAware) — evict the least
  visible entry first (spec §5.19 *visibility priority*), then fall back to
  recency. This is the default.
