Sets the memory budget the decoded-image cache is held under, evicting down
to it immediately, and returns how many images that evicted (spec §5.19
*memory budgets*).

The default is [`CacheBudget::unlimited`]: every decoded image stays
resident for the life of the runtime, which is what a host that never calls
this gets. Passing a byte or entry cap makes the cache evict by the
installed [`EvictionPolicy`] — least visible first, then least recently
looked up — and an evicted image reloads when a later frame references it
again. A budget smaller than the frame's live set therefore trades memory
for reloads; that is the caller's choice to make.

[`CacheBudget::disabled`] admits nothing. For this cache that means no
image is ever resident and none paints — it is the residency store, not a
memo of something compose could recompute, so there is no uncached path to
fall back to.
