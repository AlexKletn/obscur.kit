Caches measure (and arranged) results per node so unchanged subtrees can
**early-exit** instead of re-measuring (spec §4.3), with a per-cache disable
switch for tests (spec §5.19).

# Constraint-keyed early exit

Each node's [`CacheEntry`] records the [`BoxConstraints`] it was measured
under. [`get_measure`](Self::get_measure) returns a hit **only when the
stored constraints exactly equal the requested constraints**; a measure
under any different constraints misses and must be recomputed. This is the
mechanism §4.3 calls "early exit when geometry is stable": a re-measure under
identical constraints reuses the cached size. Because constraints hold `f32`,
the comparison is exact (`PartialEq`) — a hit requires bitwise-identical
constraint values.

# Disable switch

A cache built with [`disabled`](Self::disabled) never stores and always
misses, so tests can exercise the uncached path deterministically (§5.19).

Because the cached results hold `f32`, this type is `PartialEq` but not `Eq`.
