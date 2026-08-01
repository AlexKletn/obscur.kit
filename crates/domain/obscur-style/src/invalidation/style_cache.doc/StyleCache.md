A cache of computed styles keyed by [`NodeId`] (§4.3, §5.19).

When enabled it stores one [`CacheEntry`] per node and serves hits from it.
When disabled every store is a no-op and every lookup misses, which is the
per-cache disable switch §5.19 requires for correctness tests. Because
[`ComputedStyle`] carries `f32` data, this derives [`PartialEq`] but not
[`Eq`].
