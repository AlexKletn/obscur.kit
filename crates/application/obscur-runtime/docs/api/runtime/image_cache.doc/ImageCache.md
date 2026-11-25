The decoded images a frame paints from, held under a memory budget with
eviction and metrics (spec §5.19 *memory budgets; eviction; visibility
priority; `DevTools` metrics*).

This is the residency store for `background-image: url(...)` and element
`src` bitmaps: [`resolve_background_images`] loads and decodes each URL once
and hands the [`SceneImage`] here, and compose paints straight out of
[`images`](Self::images). Decoded pixels are the runtime's largest per-URL
cost — `width × height × 4` bytes each — so this is where an unbounded map
would grow a long-running app without limit.

# What the §5.19 policy types decide here

The map of values is this type's own; the *decisions* are the domain's:
[`CacheBudget`] rules on admission and over-budget, [`EvictionPolicy`] picks
the victim from the [`CacheEntry`] bookkeeping, and [`CacheStatistics`] is the
live byte/entry total as well as the `DevTools` hit/miss/eviction counters.

# Eviction is visible to the caller

[`insert`](Self::insert) and [`set_budget`](Self::set_budget) **return** the
evicted URLs rather than dropping them silently. The runtime must act on that
list: its `image_handles` map is the "already requested, do not retry" guard,
so an evicted URL that stayed in it would never load again and the image would
silently stop painting. Returning the list is what keeps eviction a cache
behaviour instead of a correctness bug.

# Thrash is the budget's own business

A budget smaller than the frame's live working set will evict an image that
the next frame references again, re-loading it — the cost of asking for a
cache smaller than what is on screen. The default budget is
[`CacheBudget::unlimited`], which admits everything and evicts nothing, so a
host that sets no budget behaves exactly as it did before this cache was
bounded.

[`resolve_background_images`]: super::UiRuntime
