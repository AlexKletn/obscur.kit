Bookkeeping metadata for one item held in a runtime cache (spec §5.19).

A cache entry records what a store needs to make eviction and metrics
decisions without holding the artifact bytes themselves: the [`CacheKey`]
identity, the artifact's `byte_size` (for memory budgets), the
[`ResourceVersion`] it was derived at (for correctness transparency — a stale
version can be invalidated), its [`VisibilityPriority`], and access
bookkeeping.

`last_access` is a **logical tick**: a monotonically increasing `u64` the
caller supplies via [`touch`](CacheEntry::touch), never a wall-clock reading.
This keeps recency deterministic (spec §5.19 caches are pure, reproducible
bookkeeping).
