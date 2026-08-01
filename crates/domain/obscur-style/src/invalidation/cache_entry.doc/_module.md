A single cached computed style with its version: [`CacheEntry`] (§4.3).

[`CacheEntry`] is the value stored per node in a [`crate::invalidation::StyleCache`]:
an owned [`ComputedStyle`] tagged with the [`VersionStamp`] it was computed
at. The stamp lets the cache reject stale hits — a lookup that asks for a newer
version than the entry holds misses, so recomputation is confined to genuinely
out-of-date nodes (§4.3).
